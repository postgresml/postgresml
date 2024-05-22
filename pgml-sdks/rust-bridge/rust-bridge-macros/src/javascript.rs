use quote::{format_ident, quote, ToTokens};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use syn::{visit::Visit, DeriveInput, ItemImpl, Type};

use crate::common::{AttributeArgs, GetImplMethod, SupportedLanguage};
use crate::types::{OutputType, SupportedType};

pub fn generate_javascript_alias(parsed: DeriveInput) -> proc_macro::TokenStream {
    let name_ident = format_ident!("{}Javascript", parsed.ident);
    let wrapped_type_ident = format_ident!("{}", parsed.ident);

    let expanded = quote! {
        #[cfg(feature = "javascript")]
        pub struct #name_ident {
            pub wrapped: std::boxed::Box<#wrapped_type_ident>
        }

        #[cfg(feature = "javascript")]
        impl From<#wrapped_type_ident> for #name_ident {
            fn from(w: #wrapped_type_ident) -> Self {
                Self {
                    wrapped: std::boxed::Box::new(w),
                }
            }
        }

        #[cfg(feature = "javascript")]
        impl From<#name_ident> for #wrapped_type_ident {
            fn from(w: #name_ident) -> Self {
                *w.wrapped
            }
        }

        #[cfg(feature = "javascript")]
        impl rust_bridge::javascript::FromJsType for #wrapped_type_ident {
            type From = neon::types::JsValue;
            fn from_js_type<'a, C: neon::context::Context<'a>>(cx: &mut C, arg: neon::handle::Handle<Self::From>) -> neon::result::NeonResult<Self> {
                let arg: neon::handle::Handle<neon::types::JsObject> = arg.downcast(cx).or_throw(cx)?;
                use neon::prelude::*;
                use core::ops::Deref;
                let s: neon::handle::Handle<neon::types::JsBox<#name_ident>> = arg.get(cx, "s")?;
                Ok(*s.wrapped.clone())
            }
        }

        #[cfg(feature = "javascript")]
        impl rust_bridge::javascript::FromJsType for &mut #wrapped_type_ident {
            type From = neon::types::JsValue;
           fn from_js_type<'a, C: neon::context::Context<'a>>(cx: &mut C, arg: neon::handle::Handle<Self::From>) -> neon::result::NeonResult<Self> {
                use neon::prelude::*;
                use core::ops::Deref;
                let arg: neon::handle::Handle<neon::types::JsObject> = arg.downcast(cx).or_throw(cx)?;
                let s: neon::handle::Handle<neon::types::JsBox<#name_ident>> = arg.get(cx, "s")?;
                unsafe {
                    let ptr = &*s.wrapped as *const #wrapped_type_ident;
                    let ptr = ptr as *mut #wrapped_type_ident;
                    let boxed = Box::from_raw(ptr);
                    Ok(Box::leak(boxed))
                }
            }
        }

        #[cfg(feature = "javascript")]
        impl rust_bridge::javascript::FromJsType for & #wrapped_type_ident {
            type From = neon::types::JsValue;
            fn from_js_type<'a, C: neon::context::Context<'a>>(cx: &mut C, arg: neon::handle::Handle<Self::From>) -> neon::result::NeonResult<Self> {
                use neon::prelude::*;
                use core::ops::Deref;
                let arg: neon::handle::Handle<neon::types::JsObject> = arg.downcast(cx).or_throw(cx)?;
                let s: neon::handle::Handle<neon::types::JsBox<#name_ident>> = arg.get(cx, "s")?;
                unsafe {
                    let ptr = &*s.wrapped as *const #wrapped_type_ident;
                    let ptr = ptr as *mut #wrapped_type_ident;
                    let boxed = Box::from_raw(ptr);
                    Ok(Box::leak(boxed))
                }
            }
        }

        #[cfg(feature = "javascript")]
        impl rust_bridge::javascript::IntoJsResult for #wrapped_type_ident {
            type Output = neon::types::JsValue;
            fn into_js_result<'a, 'b, 'c: 'b, C: neon::context::Context<'c>>(self, cx: &mut C) -> neon::result::JsResult<'b, Self::Output> {
                use rust_bridge::javascript::IntoJsResult;
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
                if !attribute_args.should_alias_method(&method_name, SupportedLanguage::JavaScript)
                {
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

        let (outer_prep_arguments, inner_prep_arguments, wrapper_arguments) =
            get_method_wrapper_arguments_javascript(&method);

        let (output_type, convert_from) = match &method.output_type {
            OutputType::Result(v) | OutputType::Other(v) => {
                convert_output_type_convert_from_javascript(v, &method)
            }
            OutputType::Default => (None, None),
        };

        let does_take_ownership_of_self = method
            .receiver
            .as_ref()
            .is_some_and(|r| r.to_string().replace("mut", "").trim() == "self");

        let p1 = method_ident.to_string();
        let p2 = method
            .method_arguments
            .iter()
            .filter(|a| !matches!(a.1, SupportedType::S))
            .map(|a| match &a.1 {
                SupportedType::Option(o) => format!(
                    "{}?: {}",
                    a.0.replace("mut", "").trim(),
                    get_typescript_type(o)
                ),
                _ => format!(
                    "{}: {}",
                    a.0.replace("mut", "").trim(),
                    get_typescript_type(&a.1)
                ),
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

        let outer_prepared = if let Some(_r) = &method.receiver {
            quote! {
                use core::ops::Deref;
                let this = cx.this().root(&mut cx);
                #(#outer_prep_arguments)*
            }
        } else {
            quote! {
                #(#outer_prep_arguments)*
            }
        };

        let inner_prepared = if let Some(_r) = &method.receiver {
            if does_take_ownership_of_self {
                quote! {
                    let this = this.into_inner(&mut cx);
                    let s: neon::handle::Handle<neon::types::JsBox<#name_ident>> = this.get(&mut cx, "s")?;
                    let wrapped = (*s.wrapped).clone();
                    #(#inner_prep_arguments)*
                }
            } else {
                quote! {
                    let this = this.into_inner(&mut cx);
                    let s: neon::handle::Handle<neon::types::JsBox<#name_ident>> = this.get(&mut cx, "s")?;
                    let wrapped = unsafe {
                        let ptr = &*s.wrapped as *const #wrapped_type_ident;
                        let ptr = ptr as *mut #wrapped_type_ident;
                        let boxed = Box::from_raw(ptr);
                        Ok(Box::leak(boxed))
                    }?;
                    #(#inner_prep_arguments)*
                }
            }
        } else {
            quote! {
                #(#inner_prep_arguments)*
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
                let x = #wrapped_call.await;
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
                    use rust_bridge::javascript::{IntoJsResult, FromJsType};

                    #outer_prepared
                    #inner_prepared

                    let channel = cx.channel();
                    let (deferred, promise) = cx.promise();
                    crate::get_or_set_runtime().spawn(async move {
                        #middle
                        deferred.try_settle_with(&channel, move |mut cx| {
                            x.into_js_result(&mut cx)
                        }).expect("Error sending js");
                    });

                    Ok(promise)
                }
            }
        } else {
            quote! {
                #signature {
                    use neon::prelude::*;
                    use rust_bridge::javascript::{IntoJsResult, FromJsType};
                    #outer_prepared
                    #inner_prepared
                    #middle
                    x.into_js_result(&mut cx)
                }
            }
        };
        methods.push(mq);
    }

    typescript_declarations.push('}');

    let path = std::env::var("TYPESCRIPT_DECLARATION_FILE");
    if let Ok(path) = path {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path)
            .unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read typescript declaration file for python");
        if !contents.contains(&format!("declare class {}", javascript_class_name)) {
            file.write_all(typescript_declarations.as_bytes())
                .expect("Unable to write typescript declaration file for python");
        }
    }
    proc_macro::TokenStream::from(quote! {
        #[cfg(feature = "javascript")]
        impl #name_ident {
            #(#methods)*
        }

        #[cfg(feature = "javascript")]
        impl rust_bridge::javascript::IntoJsResult for #name_ident {
            type Output = neon::types::JsValue;
            fn into_js_result<'a, 'b, 'c: 'b, C: neon::context::Context<'c>>(self, cx: &mut C) -> neon::result::JsResult<'b, Self::Output> {
                use neon::object::Object;
                use neon::prelude::Value;
                let obj = cx.empty_object();
                let s = cx.boxed(self);
                obj.set(cx, "s", s)?;
                #(#object_sets)*
                Ok(obj.as_value(cx))
            }
        }

        #[cfg(feature = "javascript")]
        impl neon::types::Finalize for #name_ident {}
    })
}

fn get_method_wrapper_arguments_javascript(
    method: &GetImplMethod,
) -> (
    Vec<proc_macro2::TokenStream>,
    Vec<proc_macro2::TokenStream>,
    Vec<proc_macro2::TokenStream>,
) {
    let mut outer_prep_arguments = Vec::new();
    let mut inner_prep_arguments = Vec::new();
    let mut method_arguments = Vec::new();

    method
        .method_arguments
        .iter()
        .enumerate()
        .for_each(|(i, (argument_name, argument_type))| {
            let argument_name_ident = format_ident!("{}", argument_name.replace("mut ", ""));
            let (outer_prep_argument, inner_prep_argument, method_argument) =
                convert_method_wrapper_arguments(argument_name_ident, argument_type, i, false);
            outer_prep_arguments.push(outer_prep_argument);
            inner_prep_arguments.push(inner_prep_argument);
            method_arguments.push(method_argument);
        });
    (outer_prep_arguments, inner_prep_arguments, method_arguments)
}

fn convert_method_wrapper_arguments(
    name_ident: syn::Ident,
    ty: &SupportedType,
    index: usize,
    checked_basic_reference: bool,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    // I think this whole piece could be done better if we fix the way we handle references, but
    // I'm not sure how to do that yet
    match (&ty, checked_basic_reference) {
        (SupportedType::Reference(r), false) => match *r.ty {
            SupportedType::str => {
                let argument_type_js = get_neon_type(&r.ty);
                let t = syn::parse_str::<syn::Type>("String")
                    .unwrap()
                    .into_token_stream();

                (
                    quote! {
                        let #name_ident = cx.argument::<#argument_type_js>(#index as i32)?;
                        let #name_ident = <#t>::from_js_type(&mut cx, #name_ident)?;
                    },
                    quote! {},
                    quote! { &#name_ident },
                )
            }
            SupportedType::i64
            | SupportedType::u64
            | SupportedType::i32
            | SupportedType::f64
            | SupportedType::bool => {
                let argument_type_js = get_neon_type(&r.ty);
                let t = r.ty.to_type(None).expect(
                    "Could not parse type in convert_method_wrapper_arguments in javascript.rs",
                );
                (
                    quote! {
                        let #name_ident = cx.argument::<#argument_type_js>(#index as i32)?;
                        let #name_ident = <#t>::from_js_type(&mut cx, #name_ident)?;
                    },
                    quote! {},
                    quote! { &#name_ident },
                )
            }
            _ => convert_method_wrapper_arguments(name_ident, ty, index, true),
        },
        (SupportedType::Option(_o), _) => {
            let t = ty.to_type(None).expect(
                "Could not parse type in convert_method_wrapper_arguments in javascript.rs",
            );
            (
                quote! {
                    let #name_ident = cx.argument_opt(#index as i32);
                    let #name_ident = <#t>::from_option_js_type(&mut cx, #name_ident)?;
                },
                quote! {},
                quote! { #name_ident },
            )
        }
        _ => {
            let argument_type_js = get_neon_type(ty);
            let t = ty.to_type(None).expect(
                "Could not parse type in convert_method_wrapper_arguments in javascript.rs",
            );
            (
                quote! {
                    let #name_ident = cx.argument::<#argument_type_js>(#index as i32)?;
                    let #name_ident = <#t>::from_js_type(&mut cx, #name_ident)?;
                },
                quote! {},
                quote! { #name_ident},
            )
        }
    }
}

fn get_neon_type(ty: &SupportedType) -> syn::Type {
    match ty {
        SupportedType::Reference(r) => get_neon_type(&r.ty),
        SupportedType::Option(o) => get_neon_type(o),
        SupportedType::str | SupportedType::String => {
            syn::parse_str("neon::types::JsString").unwrap()
        }
        SupportedType::bool => syn::parse_str("neon::types::JsBoolean").unwrap(),
        SupportedType::Vec(_v) => syn::parse_str("neon::types::JsArray").unwrap(),
        SupportedType::S => syn::parse_str("neon::types::JsObject").unwrap(),
        SupportedType::Tuple(_t) => syn::parse_str("neon::types::JsObject").unwrap(),
        SupportedType::HashMap((_k, _v)) => syn::parse_str("neon::types::JsObject").unwrap(),
        SupportedType::i64 | SupportedType::f64 | SupportedType::u64 => {
            syn::parse_str("neon::types::JsNumber").unwrap()
        }
        SupportedType::CustomType(_t) => syn::parse_str("neon::types::JsValue").unwrap(),
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
            Some(quote! {neon::result::JsResult<'a, neon::types::JsValue>}),
            Some(format_ident!("Self").into_token_stream()),
        ),
        // t @ SupportedType::Database
        // | t @ SupportedType::PipelineSyncData
        // | t @ SupportedType::Model
        // | t @ SupportedType::Splitter
        // | t @ SupportedType::Collection => (
        //     Some(quote! {neon::result::JsResult<'a, neon::types::JsObject>}),
        //     Some(format_ident!("{}Javascript", t.to_string()).into_token_stream()),
        // ),
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

        SupportedType::CustomType(t) => t.to_string(),
        // Add more types as required
        _ => "any".to_string(),
    }
}
