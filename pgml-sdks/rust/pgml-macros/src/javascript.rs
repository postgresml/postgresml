use quote::{format_ident, quote, ToTokens};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use syn::{visit::Visit, DeriveInput, ItemImpl, Type};

use crate::common::{AttributeArgs, GetImplMethod};
use crate::types::{GetSupportedType, OutputType, SupportedType};

pub fn generate_custom_into_js_result(parsed: DeriveInput) -> proc_macro::TokenStream {
    let name = parsed.ident;
    let fields_named = match parsed.data {
        syn::Data::Struct(s) => match s.fields {
            syn::Fields::Named(n) => n,
            _ => panic!("custom_into_js proc_macro structs should only have named fields"),
        },
        _ => panic!("custom_into_js proc_macro should only be used on structs"),
    };

    let mut sets = Vec::new();
    let mut interface = format!("\ninterface {} {{\n", name);

    fields_named.named.into_pairs().for_each(|p| {
        let v = p.into_value();
        let name = v.ident.to_token_stream().to_string();
        let name_ident = v.ident;
        sets.push(quote! {
            let js_item = self.#name_ident.into_js_result(cx)?;
            js_object.set(cx, #name, js_item)?;
        });
        let ty = GetSupportedType::get_type(&v.ty);
        let decleration = match &ty {
            SupportedType::Option(o) => format!("{}?", get_typescript_type(o)),
            _ => get_typescript_type(&ty),
        };
        interface.push_str(&format!("\t{}: {},\n", name, decleration));
    });

    interface.push('}');
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .read(true)
        .open("javascript/index.d.ts")
        .unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read typescript decleration file");
    if !contents.contains(&interface) {
        file.write_all(interface.as_bytes())
            .expect("Unable to write typescript decleration file");
    }

    let out = quote! {
        #[cfg(feature = "javascript")]
        impl IntoJsResult for #name {
            type Output = neon::types::JsObject;
            fn into_js_result<'a, 'b, 'c: 'b, C: neon::context::Context<'c>>(self, cx: &mut C) -> neon::result::JsResult<'b, Self::Output> {
                use neon::object::Object;
                let js_object = cx.empty_object();
                #(#sets)*
                Ok(js_object)
            }
        }
    };
    proc_macro::TokenStream::from(out)
}

pub fn generate_javascript_derive(parsed: DeriveInput) -> proc_macro::TokenStream {
    let name_ident = format_ident!("{}Javascript", parsed.ident);
    let wrapped_type_ident = format_ident!("{}", parsed.ident);

    let expanded = quote! {
        #[cfg(feature = "javascript")]
        pub struct #name_ident {
            wrapped: #wrapped_type_ident
        }

        #[cfg(feature = "javascript")]
        impl From<#wrapped_type_ident> for #name_ident {
            fn from(w: #wrapped_type_ident) -> Self {
                Self {
                    wrapped: w,
                }
            }
        }

        #[cfg(feature = "javascript")]
        impl From<#name_ident> for #wrapped_type_ident {
            fn from(w: #name_ident) -> Self {
                w.wrapped
            }
        }

        #[cfg(feature = "javascript")]
        impl FromJsType for #name_ident {
            type From = neon::types::JsObject;
            fn from_js_type<'a, C: neon::context::Context<'a>>(cx: &mut C, arg: neon::handle::Handle<Self::From>) -> neon::result::NeonResult<Self> {
                use neon::prelude::*;
                use core::ops::Deref;
                let s: neon::handle::Handle<neon::types::JsBox<std::cell::RefCell<#name_ident>>> = arg.get(cx, "s")?;
                let wrapped = (*s).deref().borrow().wrapped.clone();
                Ok(Self {
                    wrapped
                })
            }
        }

        #[cfg(feature = "javascript")]
        impl IntoJsResult for #wrapped_type_ident {
            type Output = neon::types::JsObject;
            fn into_js_result<'a, 'b, 'c: 'b, C: neon::context::Context<'c>>(self, cx: &mut C) -> neon::result::JsResult<'b, Self::Output> {
                #name_ident::from(self).into_js_result(cx)
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

pub fn generate_javascript_methods(
    parsed: ItemImpl,
    attribute_args: &AttributeArgs,
) -> proc_macro::TokenStream {
    let mut methods = Vec::new();
    let mut object_sets = Vec::new();

    let wrapped_type_ident = match *parsed.self_ty {
        Type::Path(p) => p.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Error getting struct ident for impl block"),
    };
    let name_ident = format_ident!("{}Javascript", wrapped_type_ident);

    let javascript_class_name = wrapped_type_ident.to_string();
    let mut typescript_declarations = format!("\ndeclare class {} {{\n", javascript_class_name);

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
        let (method_arguments, wrapper_arguments) =
            get_method_wrapper_arguments_javascript(&method);
        let (output_type, convert_from) = match &method.output_type {
            OutputType::Result(v) | OutputType::Other(v) => {
                convert_output_type_convert_from_javascript(v, &method)
            }
            OutputType::Default => (None, None),
        };

        let p1 = method_ident.to_string();
        let p2 = method
            .method_arguments
            .iter()
            .filter(|a| !matches!(a.1, SupportedType::S))
            .map(|a| match &a.1 {
                SupportedType::Option(o) => format!("{}?: {}", a.0, get_typescript_type(o)),
                _ => format!("{}: {}", a.0, get_typescript_type(&a.1)),
            })
            .collect::<Vec<String>>()
            .join(", ");
        let p3 = match &method.output_type {
            OutputType::Result(v) | OutputType::Other(v) => match v {
                SupportedType::S => wrapped_type_ident.to_string(),
                _ => get_typescript_type(v),
            },
            OutputType::Default => "void".to_string(),
        };
        if method.is_async {
            typescript_declarations.push_str(&format!("\n\t{}({}): Promise<{}>;\n", p1, p2, p3));
        } else {
            typescript_declarations.push_str(&format!("\n\t{}({}): {};\n", p1, p2, p3));
        }

        let method_name_string = method_ident.to_string();
        object_sets.push(quote! {
            let f: neon::handle::Handle<neon::types::JsFunction> = neon::types::JsFunction::new(cx, #name_ident::#method_ident)?;
            obj.set(cx, #method_name_string, f)?;
        });

        let signature = quote! {
            pub fn #method_ident<'a>(mut cx: neon::context::FunctionContext<'a>) -> #output_type
        };
        let prep_arguments = if let Some(_r) = &method.receiver {
            quote! {
                use core::ops::Deref;
                let this = cx.this();
                let s: neon::handle::Handle<neon::types::JsBox<std::cell::RefCell<#name_ident>>> = this.get(&mut cx, "s")?;
                let wrapped = (*s).deref().borrow();
                let wrapped = wrapped.wrapped.clone();
                #(#method_arguments)*
            }
        } else {
            quote! {
                #(#method_arguments)*
            }
        };

        let wrapped_call = if method_name_string == "new" {
            quote! {
                #wrapped_type_ident::new(#(#wrapper_arguments),*)
            }
        } else {
            quote! {
                wrapped.#method_ident(#(#wrapper_arguments),*)
            }
        };

        let middle = if method.is_async {
            quote! {
                let runtime = crate::get_or_set_runtime();
                let x = runtime.block_on(#wrapped_call);

            }
        } else {
            quote! {
                let x = #wrapped_call;
            }
        };
        let middle = if let OutputType::Result(_) = method.output_type {
            quote! {
                #middle
                let x = x.expect("Error in rust method");
            }
        } else {
            middle
        };
        let middle = if let Some(convert) = convert_from {
            quote! {
                #middle
                let x = #convert::from(x);
            }
        } else {
            middle
        };
        let mq = if method.is_async {
            quote! {
                #signature {
                    use neon::prelude::*;
                    #prep_arguments
                    let channel = cx.channel();
                    let (deferred, promise) = cx.promise();
                    deferred.try_settle_with(&channel, move |mut cx| {
                        #middle
                        x.into_js_result(&mut cx)
                    }).expect("Error sending js");
                    Ok(promise)
                }
            }
        } else {
            quote! {
                #signature {
                    use neon::prelude::*;
                    #prep_arguments
                    #middle
                    x.into_js_result(&mut cx)
                }
            }
        };
        methods.push(mq);
    }

    typescript_declarations.push('}');

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .read(true)
        .open("javascript/index.d.ts")
        .unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read typescript declaration file for python");
    if !contents.contains(&format!("declare class {}", javascript_class_name)) {
        file.write_all(typescript_declarations.as_bytes())
            .expect("Unable to write typescript declaration file for python");
    }

    proc_macro::TokenStream::from(quote! {
        #[cfg(feature = "javascript")]
        impl #name_ident {
            #(#methods)*
        }

        #[cfg(feature = "javascript")]
        impl IntoJsResult for #name_ident {
            type Output = neon::types::JsObject;
            fn into_js_result<'a, 'b, 'c: 'b, C: neon::context::Context<'c>>(self, cx: &mut C) -> neon::result::JsResult<'b, Self::Output> {
                use neon::object::Object;
                let obj = cx.empty_object();
                let s = cx.boxed(std::cell::RefCell::new(self));
                obj.set(cx, "s", s)?;
                #(#object_sets)*
                Ok(obj)
            }
        }

        #[cfg(feature = "javascript")]
        impl neon::types::Finalize for #name_ident {}
    })
}

fn get_method_wrapper_arguments_javascript(
    method: &GetImplMethod,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    let mut wrapper_arguments = Vec::new();
    let mut method_arguments = Vec::new();

    method
        .method_arguments
        .iter()
        .enumerate()
        .for_each(|(i, (_argument_name, argument_type))| {
            let argument_ident = format_ident!("arg{}", i);
            let (argument_type_tokens, wrapper_argument_tokens) = convert_method_wrapper_arguments(
                argument_ident.clone(),
                argument_type,
            );
            let argument_type_js = get_neon_type(argument_type);
            let method_argument = match argument_type {
                SupportedType::Option(_o) => quote! {
                    let #argument_ident = cx.argument_opt(#i as i32);
                    let #argument_ident = <#argument_type_tokens>::from_option_js_type(&mut cx, #argument_ident)?;
                },
                _ => quote! {
                    let #argument_ident = cx.argument::<#argument_type_js>(#i as i32)?;
                    let #argument_ident = <#argument_type_tokens>::from_js_type(&mut cx, #argument_ident)?;
                }
            };
            method_arguments.push(method_argument);
            wrapper_arguments.push(wrapper_argument_tokens.into_token_stream());
        });

    (method_arguments, wrapper_arguments)
}

fn convert_method_wrapper_arguments(
    name_ident: syn::Ident,
    ty: &SupportedType,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    match ty {
        SupportedType::Reference(r) => {
            let (d, w) = convert_method_wrapper_arguments(name_ident, &r.ty);
            (d, quote! { & #w})
        }
        SupportedType::str => (
            syn::parse_str::<syn::Type>("String")
                .unwrap()
                .into_token_stream(),
            quote! { #name_ident},
        ),
        _ => {
            let t = ty.to_type(Some("Javascript")).expect(
                "Could not parse type in convert_method_wrapper_arguments in javascript.rs",
            );
            (t.into_token_stream(), quote! {#name_ident.into()})
        }
    }
}

fn get_neon_type(ty: &SupportedType) -> syn::Type {
    match ty {
        SupportedType::Reference(r) => get_neon_type(&r.ty),
        SupportedType::str | SupportedType::String => {
            syn::parse_str("neon::types::JsString").unwrap()
        }
        SupportedType::bool => syn::parse_str("neon::types::JsBoolean").unwrap(),
        SupportedType::Vec(_v) => syn::parse_str("neon::types::JsArray").unwrap(),
        SupportedType::S => syn::parse_str("neon::types::JsObject").unwrap(),
        SupportedType::DateTime => syn::parse_str("neon::types::JsDate").unwrap(),
        SupportedType::Tuple(_t) => syn::parse_str("neon::types::JsObject").unwrap(),
        SupportedType::HashMap((_k, _v)) => syn::parse_str("neon::types::JsObject").unwrap(),
        SupportedType::i64 | SupportedType::f64 | SupportedType::u64 => {
            syn::parse_str("neon::types::JsNumber").unwrap()
        }
        // Our own types
        SupportedType::Database
        | SupportedType::Collection
        | SupportedType::Splitter
        | SupportedType::QueryBuilder
        | SupportedType::QueryRunner
        | SupportedType::Model => syn::parse_str("neon::types::JsObject").unwrap(),
        // Add more types as required
        _ => syn::parse_str("neon::types::JsValue").unwrap(),
    }
}

fn convert_output_type_convert_from_javascript(
    ty: &SupportedType,
    method: &GetImplMethod,
) -> (
    Option<proc_macro2::TokenStream>,
    Option<proc_macro2::TokenStream>,
) {
    let (output_type, convert_from) = match ty {
        SupportedType::S => (
            Some(quote! {neon::result::JsResult<'a, neon::types::JsObject>}),
            Some(format_ident!("Self").into_token_stream()),
        ),
        t @ SupportedType::Database
        | t @ SupportedType::Model
        | t @ SupportedType::Splitter
        | t @ SupportedType::Collection => (
            Some(quote! {neon::result::JsResult<'a, neon::types::JsObject>}),
            Some(format_ident!("{}Javascript", t.to_string()).into_token_stream()),
        ),
        t => {
            let ty = get_neon_type(t);
            (Some(quote! {neon::result::JsResult<'a, #ty>}), None)
        }
    };

    if method.is_async {
        (
            Some(quote! {neon::result::JsResult<'a, neon::types::JsPromise>}),
            convert_from,
        )
    } else {
        (output_type, convert_from)
    }
}

fn get_typescript_type(ty: &SupportedType) -> String {
    match ty {
        SupportedType::Reference(r) => get_typescript_type(&r.ty),
        SupportedType::str | SupportedType::String => "string".to_string(),
        SupportedType::bool => "boolean".to_string(),
        SupportedType::Option(o) => get_typescript_type(o),
        SupportedType::Vec(v) => format!("{}[]", get_typescript_type(v)),
        SupportedType::HashMap((k, v)) => {
            format!(
                "Map<{}, {}>",
                get_typescript_type(k),
                get_typescript_type(v)
            )
        }
        SupportedType::JsonHashMap => "Map<string, string>".to_string(),
        SupportedType::DateTime => "Date".to_string(),
        SupportedType::Tuple(t) => {
            let mut types = Vec::new();
            for ty in t {
                types.push(get_typescript_type(ty));
            }
            // Rust's unit type is represented as an empty tuple
            if types.is_empty() {
                "void".to_string()
            } else {
                format!("[{}]", types.join(", "))
            }
        }
        SupportedType::i64 | SupportedType::f64 | SupportedType::u64 => "number".to_string(),
        // Our own types
        t @ SupportedType::Database
        | t @ SupportedType::Collection
        | t @ SupportedType::Splitter
        | t @ SupportedType::QueryBuilder
        | t @ SupportedType::QueryRunner
        | t @ SupportedType::Model => t.to_string(),
        // Add more types as required
        _ => "any".to_string(),
    }
}
