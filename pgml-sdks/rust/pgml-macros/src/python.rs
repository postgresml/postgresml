use quote::{format_ident, quote, ToTokens};
use syn::{visit::Visit, DeriveInput, ItemImpl, Type};

use crate::common::{AttributeArgs, GetImplMethod};
use crate::types::{GetSupportedType, OutputType, SupportedType};

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
        let ty = GetSupportedType::get_type(&v.ty);
        let adjusted = match ty {
            SupportedType::Json(_j) => quote! { self.#name_ident.0},
            SupportedType::DateTime(_d) => quote! { self.#name_ident.timestamp()},
            _ => quote! {self.#name_ident}
        };
        quote! {
            dict.set_item(#name, #adjusted).expect("Error setting python value in custom_into_py proc_macro");
        }
    }).collect();

    let expanded = quote! {
        impl IntoPy<PyObject> for #name {
            fn into_py(self, py: Python<'_>) -> PyObject {
                let dict = PyDict::new(py);
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
        #[pyclass(name = #wrapped_type_name)]
        #[derive(Debug)]
        pub struct #name_ident {
            wrapped: #wrapped_type_ident
        }

        impl From<#wrapped_type_ident> for #name_ident {
            fn from(w: #wrapped_type_ident) -> Self {
                Self {
                    wrapped: w,
                }
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

        // The new function for pyO3 requires some unique syntax
        let (signature, middle) = if method_ident.to_string() == "new" {
            let signature = quote! {
                #[new]
                pub fn new<'a>(#(#method_arguments),*) -> #output_type
            };
            let middle = quote! {
                     let runtime = get_or_set_runtime();
                     let x = match runtime.block_on(#wrapped_type_ident::new(#(#wrapper_arguments),*)) {
                        Ok(m) => m,
                        Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string()))

                     };
                     Ok(#name_ident::from(x))
                 };
            (signature, middle)
        } else {
            let signature = quote! {
                pub fn #method_ident<'a>(#(#method_arguments),*) -> #output_type
            };
            let middle = quote! {
                #method_ident(#(#wrapper_arguments),*)
            };
            let middle = if method.is_async {
                quote! {
                    wrapped.#middle.await
                }
            } else {
                quote! {
                    self.wrapped.#middle
                }
            };
            let middle = if let OutputType::Result(_r) = method.output_type {
                quote! {
                    let x = match #middle {
                        Ok(m) => m,
                        Err(e) => return Err(PyErr::new::<pyo3::exceptions::PyException, _>(e.to_string()))
                    };
                }
            } else {
                quote! {
                    let x = middle;
                }
            };
            let middle = if let Some(convert) = convert_from {
                quote! {
                    #middle
                    let x = #convert::from(x);
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

    proc_macro::TokenStream::from(quote! {
        #[pymethods]
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

    if let Some(receiver) = &method.receiver {
        method_arguments.push(receiver.clone());
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
        py: Python<'a>
    };
    if method_arguments.len() > 0 {
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
                .to_type()
                .expect("Could not parse type in convert_method_type in python.rs");
            (quote! { #name_ident : #t}, quote! {#name_ident})
        }
    }
}

pub fn convert_output_type_convert_from_python(
    ty: &SupportedType,
    method: &GetImplMethod,
) -> (
    Option<proc_macro2::TokenStream>,
    Option<proc_macro2::TokenStream>,
) {
    let (output_type, convert_from) = match ty {
        SupportedType::S => (
            Some(quote! {PyResult<Self>}),
            Some(format_ident!("{}", method.method_ident).into_token_stream()),
        ),
        t @ SupportedType::Database | t @ SupportedType::Collection => (
            Some(quote! {PyResult<&'a PyAny>}),
            Some(format_ident!("{}Python", t.to_string()).into_token_stream()),
        ),
        t @ _ => {
            let ty = t
                .to_type()
                .expect("Error converting to type in convert_output_type_convert_from_python");
            (Some(quote! {PyResult<#ty>}), None)
        }
    };

    if method.is_async && method.method_ident.to_string() != "new" {
        (Some(quote! {PyResult<&'a PyAny>}), convert_from)
    } else {
        (output_type, convert_from)
    }
}
