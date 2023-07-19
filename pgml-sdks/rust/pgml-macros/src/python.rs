use quote::{format_ident, quote, ToTokens};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use syn::{visit::Visit, DeriveInput, ItemImpl, Type};

use crate::common::{AttributeArgs, GetImplMethod};
use crate::types::{OutputType, SupportedType};

const STUB_TOP: &str = r#"
# Top of file key: A12BECOD!
from typing import List, Dict, Optional, Self, Any

"#;

pub fn generate_into_py(parsed: DeriveInput) -> proc_macro::TokenStream {
    let name = parsed.ident;
    let fields_named = match parsed.data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(n) => n,
            _ => panic!("custom_into_py proc_macro structs should only have named fields"),
        },
        _ => panic!("custom_into_py proc_macro should only be used on structs"),
    };

    let sets: Vec<proc_macro2::TokenStream> = fields_named.named.into_pairs().map(|p| {
        let v = p.into_value();
        let name = v.ident.to_token_stream().to_string();
        let name_ident = v.ident;
        quote! {
            dict.set_item(#name, self.#name_ident).expect("Error setting python value in custom_into_py proc_macro");
        }
    }).collect();

    let stub = format!("\n{} = dict[str, Any]\n", name);

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .read(true)
        .open("python/pgml/pgml.pyi")
        .unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read stubs file for python");
    if !contents.contains(&stub) {
        file.write_all(stub.as_bytes())
            .expect("Unable to write stubs file for python");
    }

    let expanded = quote! {
        #[cfg(feature = "python")]
        impl pyo3::conversion::IntoPy<pyo3::PyObject> for #name {
            fn into_py(self, py: pyo3::marker::Python<'_>) -> pyo3::PyObject {
                let dict = pyo3::types::PyDict::new(py);
                #(#sets)*
                dict.into()
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

pub fn generate_python_derive(parsed: DeriveInput) -> proc_macro::TokenStream {
    let name_ident = format_ident!("{}Python", parsed.ident);
    let wrapped_type_ident = parsed.ident;
    let wrapped_type_name = wrapped_type_ident.to_string();
    // May also want to put a __print__ method here (if that works) automatically for every CustomDerive struct
    let expanded = quote! {
        #[cfg(feature = "python")]
        #[pyo3::pyclass(name = #wrapped_type_name)]
        #[derive(Debug, Clone)]
        pub struct #name_ident {
            wrapped: #wrapped_type_ident
        }

        #[cfg(feature = "python")]
        impl From<#wrapped_type_ident> for #name_ident {
            fn from(w: #wrapped_type_ident) -> Self {
                Self {
                    wrapped: w,
                }
            }
        }

        #[cfg(feature = "python")]
        impl From<#name_ident> for #wrapped_type_ident {
            fn from(w: #name_ident) -> Self {
                w.wrapped
            }
        }

        #[cfg(feature = "python")]
        impl pyo3::IntoPy<pyo3::PyObject> for #wrapped_type_ident {
            fn into_py(self, py: pyo3::Python) -> pyo3::PyObject {
                use pyo3::conversion::IntoPy;
                #name_ident::from(self.clone()).into_py(py)
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

pub fn generate_python_methods(
    parsed: ItemImpl,
    attribute_args: &AttributeArgs,
) -> proc_macro::TokenStream {
    let mut methods = Vec::new();

    let wrapped_type_ident = match *parsed.self_ty {
        Type::Path(p) => p.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Error getting struct ident for impl block"),
    };
    let name_ident = format_ident!("{}Python", wrapped_type_ident);

    let python_class_name = wrapped_type_ident.to_string();
    let mut stubs = format!("\nclass {}:\n", python_class_name);

    // Iterate over the items - see: https://docs.rs/syn/latest/syn/enum.ImplItem.html
    for item in parsed.items {
        // We only create methods for functions listed in the attribute args
        match &item {
            syn::ImplItem::Fn(f) => {
                let method_name = f.sig.ident.to_string();
                if !attribute_args.args.contains(&method_name) {
                    continue;
                }
            }
            _ => continue,
        }

        // Get ImplMethod details - see: https://docs.rs/syn/latest/syn/visit/index.html
        let mut method = GetImplMethod::default();
        method.visit_impl_item(&item);
        if !method.exists {
            continue;
        }
        let method_ident = method.method_ident.clone();

        let (method_arguments, wrapper_arguments) = get_method_wrapper_arguments_python(&method);
        let (output_type, convert_from) = match &method.output_type {
            OutputType::Result(v) | OutputType::Other(v) => {
                convert_output_type_convert_from_python(v, &method)
            }
            OutputType::Default => (None, None),
        };

        let signature = quote! {
            pub fn #method_ident<'a>(#(#method_arguments),*) -> #output_type
        };

        let p1 = if method.is_async { "async def" } else { "def" };
        let p2 = match method_ident.to_string().as_str() {
            "new" => "__init__".to_string(),
            _ => method_ident.to_string(),
        };
        let p3 = method
            .method_arguments
            .iter()
            .map(|a| format!("{}: {}", a.0, get_python_type(&a.1)))
            .collect::<Vec<String>>()
            .join(", ");
        let p4 = match &method.output_type {
            OutputType::Result(v) | OutputType::Other(v) => get_python_type(v),
            OutputType::Default => "None".to_string(),
        };
        stubs.push_str(&format!("\t{} {}(self, {}) -> {}", p1, p2, p3, p4));
        stubs.push_str("\n\t\t...\n");

        // The new function for pyO3 requires some unique syntax
        let (signature, middle) = if method_ident == "new" {
            let signature = quote! {
                #[new]
                #signature
            };
            let middle = if method.is_async {
                quote! {
                    crate::get_or_set_runtime().block_on(#wrapped_type_ident::new(#(#wrapper_arguments),*))
                }
            } else {
                quote! {
                    #wrapped_type_ident::new(#(#wrapper_arguments),*)
                }
            };
            let middle = if let OutputType::Result(_r) = method.output_type {
                quote! {
                    let x = match #middle {
                        Ok(m) => m,
                        Err(e) => return Err(pyo3::PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string()))
                    };
                }
            } else {
                quote! {
                    let x = #middle;
                }
            };
            let middle = quote! {
                #middle
                Ok(#name_ident::from(x))
            };
            (signature, middle)
        } else {
            let middle = quote! {
                #method_ident(#(#wrapper_arguments),*)
            };
            let middle = if method.is_async {
                quote! {
                    wrapped.#middle.await
                }
            } else {
                quote! {
                    wrapped.#middle
                }
            };
            let middle = if let OutputType::Result(_r) = method.output_type {
                quote! {
                    let x = match #middle {
                        Ok(m) => m,
                        Err(e) => return Err(pyo3::PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string()))
                    };
                }
            } else {
                quote! {
                    let x = #middle;
                }
            };
            let middle = if let Some(convert) = convert_from {
                quote! {
                    #middle
                    let x = <#convert>::from(x);
                }
            } else {
                middle
            };
            let middle = if method.is_async {
                quote! {
                    let wrapped = self.wrapped.clone();
                    pyo3_asyncio::tokio::future_into_py(py, async move {
                        #middle
                        Ok(x)
                    })
                }
            } else {
                quote! {
                    let wrapped = self.wrapped.clone();
                    #middle
                    Ok(x)
                }
            };
            (signature, middle)
        };

        methods.push(quote! {
            #signature {
                #middle
            }
        });
    }

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .read(true)
        .open("python/pgml/pgml.pyi")
        .unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read stubs file for python");
    if !contents.contains("A12BECOD") {
        file.write_all(STUB_TOP.as_bytes())
            .expect("Unable to write stubs file for python");
    }
    if !contents.contains(&format!("class {}:", python_class_name)) {
        file.write_all(stubs.as_bytes())
            .expect("Unable to write stubs file for python");
    }

    proc_macro::TokenStream::from(quote! {
        #[cfg(feature = "python")]
        #[pyo3::pymethods]
        impl #name_ident {
            #(#methods)*
        }
    })
}

pub fn get_method_wrapper_arguments_python(
    method: &GetImplMethod,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    let mut method_arguments = Vec::new();
    let mut wrapper_arguments = Vec::new();

    if let Some(_receiver) = &method.receiver {
        method_arguments.push(quote! { &self });
    }

    method
        .method_arguments
        .iter()
        .for_each(|(argument_name, argument_type)| {
            let argument_name_ident = format_ident!("{}", argument_name.replace("mut ", ""));
            let (method_argument, wrapper_argument) =
                convert_method_wrapper_arguments(argument_name_ident, argument_type);
            method_arguments.push(method_argument);
            wrapper_arguments.push(wrapper_argument);
        });

    let extra_arg = quote! {
        py: pyo3::Python<'a>
    };
    if !method_arguments.is_empty() {
        method_arguments.insert(1, extra_arg);
    } else {
        method_arguments.push(extra_arg);
    }

    (method_arguments, wrapper_arguments)
}

fn convert_method_wrapper_arguments(
    name_ident: syn::Ident,
    ty: &SupportedType,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    match ty {
        SupportedType::Reference(r) => {
            let (d, w) = convert_method_wrapper_arguments(name_ident, r);
            (d, quote! { & #w})
        }
        SupportedType::str => (quote! {#name_ident: String}, quote! { #name_ident}),
        _ => {
            let t = ty
                .to_type(Some("Python"))
                .expect("Could not parse type in convert_method_type in python.rs");
            (quote! { #name_ident : #t}, quote! {#name_ident.into()})
        }
    }
}

fn convert_output_type_convert_from_python(
    ty: &SupportedType,
    method: &GetImplMethod,
) -> (
    Option<proc_macro2::TokenStream>,
    Option<proc_macro2::TokenStream>,
) {
    let (output_type, convert_from) = match ty {
        SupportedType::S => (
            Some(quote! {pyo3::PyResult<Self>}),
            Some(format_ident!("Self").into_token_stream()),
        ),
        t => {
            let ty = t
                .to_type(Some("Python"))
                .expect("Error converting to type in convert_output_type_convert_from_python");
            (Some(quote! {pyo3::PyResult<#ty>}), Some(quote! {#ty}))
        }
    };

    if method.is_async && method.method_ident != "new" {
        (Some(quote! {pyo3::PyResult<&'a pyo3::PyAny>}), convert_from)
    } else {
        (output_type, convert_from)
    }
}

fn get_python_type(ty: &SupportedType) -> String {
    match ty {
        SupportedType::Reference(r) => get_python_type(r),
        SupportedType::S => "Self".to_string(),
        SupportedType::str | SupportedType::String => "str".to_string(),
        SupportedType::bool => "bool".to_string(),
        SupportedType::Option(o) => format!(
            "Optional[{}] = {}",
            get_python_type(o),
            get_type_for_optional(o)
        ),
        SupportedType::Vec(v) => format!("List[{}]", get_python_type(v)),
        SupportedType::HashMap((k, v)) => {
            format!("Dict[{}, {}]", get_python_type(k), get_python_type(v))
        }
        SupportedType::Tuple(t) => {
            let mut types = Vec::new();
            for ty in t {
                types.push(get_python_type(ty));
            }
            // Rust's unit type is represented as an empty tuple
            if types.is_empty() {
                "None".to_string()
            } else {
                format!("tuple[{}]", types.join(", "))
            }
        }
        SupportedType::i64 => "int".to_string(),
        SupportedType::f64 => "float".to_string(),
        // Our own types
        t @ SupportedType::Database
        | t @ SupportedType::Collection
        | t @ SupportedType::Model
        | t @ SupportedType::QueryBuilder
        | t @ SupportedType::QueryRunner
        | t @ SupportedType::Splitter => t.to_string(),
        // Add more types as required
        _ => "Any".to_string(),
    }
}

fn get_type_for_optional(ty: &SupportedType) -> String {
    match ty {
        SupportedType::Reference(r) => get_type_for_optional(r),
        SupportedType::str | SupportedType::String => {
            "\"Default set in Rust. Please check the documentation.\"".to_string()
        }
        SupportedType::HashMap(_) => "{}".to_string(),
        SupportedType::Vec(_) => "[]".to_string(),
        SupportedType::i64 => 1.to_string(),
        SupportedType::f64 => 1.0.to_string(),
        SupportedType::Json => "{}".to_string(),
        _ => panic!("Type not yet supported for optional python stub: {:?}", ty),
    }
}
