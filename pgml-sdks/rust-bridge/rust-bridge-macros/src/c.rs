use proc_macro2::Ident;
use quote::{format_ident, quote};
use std::str::FromStr;
use syn::{visit::Visit, DeriveInput, ItemImpl, Type};

use crate::{
    common::{AttributeArgs, GetImplMethod, SupportedLanguage},
    types::{OutputType, SupportedType},
};

pub fn generate_c_alias(parsed: DeriveInput) -> proc_macro::TokenStream {
    let name_ident = format_ident!("{}C", parsed.ident);
    let wrapped_type_ident = parsed.ident;

    let expanded = quote! {
        #[cfg(feature = "c")]
        pub struct #name_ident {
            pub wrapped: #wrapped_type_ident
        }

        #[cfg(feature = "c")]
        unsafe impl rust_bridge::c::CustomInto<*mut #name_ident> for #wrapped_type_ident {
            unsafe fn custom_into(self) -> *mut #name_ident {
                Box::into_raw(Box::new(
                    #name_ident {
                        wrapped: self
                    }
                ))
            }
        }

        #[cfg(feature = "c")]
        unsafe impl rust_bridge::c::CustomInto<#wrapped_type_ident> for *mut #name_ident {
            unsafe fn custom_into(self) -> #wrapped_type_ident {
                let c = Box::from_raw(self);
                c.wrapped
            }
        }

        #[cfg(feature = "c")]
        unsafe impl rust_bridge::c::CustomInto<&'static mut #wrapped_type_ident> for *mut #name_ident {
            unsafe fn custom_into(self) -> &'static mut #wrapped_type_ident {
                let c = Box::leak(Box::from_raw(self));
                &mut c.wrapped
            }
        }

        #[cfg(feature = "c")]
        unsafe impl rust_bridge::c::CustomInto<&'static #wrapped_type_ident> for *mut #name_ident {
            unsafe fn custom_into(self) -> &'static #wrapped_type_ident {
                let c = Box::leak(Box::from_raw(self));
                &c.wrapped
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

pub fn generate_c_methods(
    parsed: ItemImpl,
    attribute_args: &AttributeArgs,
) -> proc_macro::TokenStream {
    let mut methods = Vec::new();

    let wrapped_type_ident = match *parsed.self_ty {
        Type::Path(p) => p.path.segments.first().unwrap().ident.clone(),
        _ => panic!("Error getting struct ident for impl block"),
    };
    let name_ident = format_ident!("{}C", wrapped_type_ident);

    for item in parsed.items {
        // We only create methods for functions listed in the attribute args
        match &item {
            syn::ImplItem::Fn(f) => {
                let method_name = f.sig.ident.to_string();
                if !attribute_args.should_alias_method(&method_name, SupportedLanguage::C) {
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

        let (mut c_function_arguments, c_argument_prep, rust_function_arguments) =
            get_method_arguments(&wrapped_type_ident, &name_ident, &method);

        let method_name = format_ident!(
            "pgml_{}_{}",
            name_ident.to_string().to_lowercase(),
            method_ident
        );

        let (return_part, augment_r_size) =
            rust_output_to_c_output(&wrapped_type_ident, &method.output_type);

        if augment_r_size {
            c_function_arguments.extend(quote! {
                , r_size: *mut std::ffi::c_ulong
            })
        }

        let async_part = if method.is_async {
            quote! { .await }
        } else {
            quote! {}
        };

        let (ret_part, augment_part) = if augment_r_size {
            (
                quote! { let (ret, ar_size) },
                quote! {*r_size = ar_size as std::ffi::c_ulong; },
            )
        } else {
            (quote! { let ret }, quote! {})
        };

        let rust_call_part = match &method.output_type {
            crate::types::OutputType::Result(_) => {
                quote! {
                    #ret_part = #wrapped_type_ident::#method_ident(#rust_function_arguments)#async_part.unwrap().custom_into();
                    #augment_part
                    ret
                }
            }
            crate::types::OutputType::Default => quote! {
                #wrapped_type_ident::#method_ident(#rust_function_arguments)#async_part;
            },
            crate::types::OutputType::Other(_) => quote! {
                #ret_part = #wrapped_type_ident::#method_ident(#rust_function_arguments)#async_part.custom_into();
                #augment_part
                ret
            },
        };

        let method = if method.is_async {
            quote! {
                #[cfg(feature = "c")]
                #[no_mangle]
                pub unsafe extern "C" fn #method_name(#c_function_arguments) #return_part {
                    use rust_bridge::c::CustomInto;
                    use rust_bridge::c::CustomIntoVec;
                    crate::get_or_set_runtime().block_on(async move {
                        #c_argument_prep
                        #rust_call_part
                    })
                }
            }
        } else {
            quote! {
                #[cfg(feature = "c")]
                #[no_mangle]
                pub unsafe extern "C" fn #method_name(#c_function_arguments) #return_part {
                    use rust_bridge::c::CustomInto;
                    use rust_bridge::c::CustomIntoVec;
                    #c_argument_prep
                    #rust_call_part
                }
            }
        };

        methods.push(method);
    }

    let method_name = format_ident!("pgml_{}_delete", name_ident.to_string().to_lowercase());
    let destructor = quote! {
        #[cfg(feature = "c")]
        #[no_mangle]
        pub unsafe extern "C" fn #method_name(ptr: *mut #name_ident) {
            drop(Box::from_raw(ptr))
        }
    };

    methods.push(destructor);

    proc_macro::TokenStream::from(quote! {
        #(#methods)*
    })
}

fn get_method_arguments(
    wrapped_type_ident: &Ident,
    name_ident: &Ident,
    method: &GetImplMethod,
) -> (
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let mut c_function_arguments = Vec::new();
    let mut c_argument_prep = Vec::new();
    let mut rust_function_arguments = Vec::new();

    if let Some(receiver) = &method.receiver {
        c_function_arguments.push(format!("s: *mut {name_ident}"));
        if receiver.to_string().contains('&') {
            c_argument_prep.push(format!(
                "let s: &mut {wrapped_type_ident} = s.custom_into();"
            ));
        } else {
            c_argument_prep.push(format!("let s: {wrapped_type_ident} = s.custom_into();"));
        }
        rust_function_arguments.push("s".to_string());
    }

    for (argument_name, argument_type) in &method.method_arguments {
        let argument_name_without_mut = argument_name.replacen("mut", "", 1);
        let (
            c_function_arguments_,
            c_function_argument_types,
            c_argument_prep_,
            rust_function_arguments_,
        ) = get_c_types(&argument_name_without_mut, argument_type);

        let c_function_arguments_ = c_function_arguments_
            .into_iter()
            .zip(c_function_argument_types)
            .map(|(argument_name, argument_type)| format!("{argument_name}: {argument_type}"))
            .collect::<Vec<String>>()
            .join(",");

        c_function_arguments.push(c_function_arguments_);
        c_argument_prep.push(c_argument_prep_);
        rust_function_arguments.push(rust_function_arguments_);
    }

    (
        proc_macro2::TokenStream::from_str(&c_function_arguments.join(",")).unwrap(),
        proc_macro2::TokenStream::from_str(&c_argument_prep.join("\n")).unwrap(),
        proc_macro2::TokenStream::from_str(&rust_function_arguments.join(",")).unwrap(),
    )
}

fn get_c_types(
    argument_name: &str,
    ty: &SupportedType,
) -> (Vec<String>, Vec<String>, String, String) {
    let t = ty.to_language_string(&None);
    let c_to_rust = format!("let {argument_name}: {t} = {argument_name}.custom_into();");
    match ty {
        SupportedType::Reference(r) => {
            let (c_function_arguments, c_function_argument_types, _, _) =
                get_c_types(argument_name, &r.ty);
            (
                c_function_arguments,
                c_function_argument_types,
                c_to_rust,
                argument_name.to_string(),
            )
        }
        SupportedType::str | SupportedType::String => (
            vec![format!("{argument_name}")],
            vec!["*mut std::ffi::c_char".to_string()],
            c_to_rust,
            argument_name.to_string(),
        ),
        SupportedType::Option(r) => {
            let (c_function_arguments, mut c_function_argument_types, _, _) =
                get_c_types(argument_name, r);

            let v = c_function_argument_types.last_mut().unwrap();
            if !v.starts_with('*') {
                *v = format!("*mut {v}");
            }

            (
                c_function_arguments,
                c_function_argument_types,
                c_to_rust,
                argument_name.to_string(),
            )
        }
        SupportedType::bool => (
            vec![format!("{argument_name}")],
            vec!["bool".to_string()],
            "".to_string(),
            argument_name.to_string(),
        ),
        SupportedType::Vec(v) => {
            let (mut c_function_arguments, mut c_function_argument_types, _, _) =
                get_c_types(argument_name, v);

            let v = c_function_argument_types.last_mut().unwrap();
            *v = v.replacen("*mut", "*mut *mut", 1);
            c_function_arguments.push("v_size".to_string());
            c_function_argument_types.push("std::ffi::c_ulong".to_string());
            let c_argument_prep = "let v_size: usize = v_size as usize;".to_string();
            let c_to_rust =
                format!("{c_argument_prep}\nlet {argument_name}: {t} = {argument_name}.custom_into_vec(v_size);");

            (
                c_function_arguments,
                c_function_argument_types,
                c_to_rust,
                argument_name.to_string(),
            )
        }
        SupportedType::HashMap(_) => panic!("HashMap arguments not supported in c"),
        SupportedType::Tuple(_) => panic!("Tuple arguments not supported in c"),
        SupportedType::S => unreachable!(),
        SupportedType::i64 => (
            vec![format!("{argument_name}")],
            vec!["std::ffi::c_long".to_string()],
            format!("let {argument_name}: {t} = {argument_name} as {t};"),
            argument_name.to_string(),
        ),
        SupportedType::u64 => (
            vec![format!("{argument_name}")],
            vec!["std::ffi::c_ulong".to_string()],
            format!("let {argument_name}: {t} = {argument_name} as {t};"),
            argument_name.to_string(),
        ),
        SupportedType::i32 => (
            vec![format!("{argument_name}")],
            vec!["std::ffi::c_int".to_string()],
            format!("let {argument_name}: {t} = {argument_name} as {t};"),
            argument_name.to_string(),
        ),
        SupportedType::f64 => (
            vec![format!("{argument_name}")],
            vec!["std::ffi::c_double".to_string()],
            format!("let {argument_name}: {t} = {argument_name} as {t};"),
            argument_name.to_string(),
        ),
        SupportedType::CustomType(s) => (
            vec![format!("{argument_name}")],
            vec![format!("*mut {s}C")],
            c_to_rust,
            argument_name.to_string(),
        ),
    }
}

fn rust_type_to_c_type(
    wrapped_type_ident: &Ident,
    ty: &SupportedType,
) -> Option<(proc_macro2::TokenStream, bool)> {
    match ty {
        // SupportedType::Reference(r) => rust_type_to_c_type(wrapped_type_ident, &r.ty),
        SupportedType::str | SupportedType::String => Some((quote! {*mut std::ffi::c_char}, false)),
        SupportedType::bool => Some((quote! { bool }, false)),
        SupportedType::Vec(v) => {
            let (ty, _) = rust_type_to_c_type(wrapped_type_ident, v).unwrap();
            Some((quote! { *mut #ty }, true))
        }
        // SupportedType::HashMap(_) => panic!("HashMap arguments not supported in c"),
        // SupportedType::Option(r) => {
        //     let mut t = get_c_types(r);
        //     if !t.0.contains('*') {
        //         t.0 = format!("*mut {}", t.0);
        //     }
        //     t
        // }
        SupportedType::Tuple(t) => {
            if !t.is_empty() {
                panic!("Tuple arguments not supported in c")
            } else {
                None
            }
        }
        SupportedType::S => {
            let ty = format_ident!("{wrapped_type_ident}C");
            Some((quote! { *mut #ty }, false))
        } // SupportedType::i64 => ("std::ffi::c_longlong".to_string(), None),
        // SupportedType::u64 => ("std::ffi::c_ulonglong".to_string(), None),
        // SupportedType::i32 => ("std::ffi::c_long".to_string(), None),
        // SupportedType::f64 => ("std::ffi::c_double".to_string(), None),
        SupportedType::CustomType(s) => {
            let ty = format_ident!("{s}C");
            Some((quote! {*mut #ty}, false))
        }
        _ => panic!("rust_type_to_c_type not implemented for {:?}", ty),
    }
}

fn rust_output_to_c_output(
    wrapped_type_ident: &Ident,
    output: &OutputType,
) -> (proc_macro2::TokenStream, bool) {
    match output {
        crate::types::OutputType::Result(r) => {
            if let Some((ty, augment_r_size)) = rust_type_to_c_type(wrapped_type_ident, r) {
                (quote! { -> #ty }, augment_r_size)
            } else {
                (quote! {}, false)
            }
        }
        crate::types::OutputType::Default => (quote! {}, false),
        crate::types::OutputType::Other(r) => {
            if let Some((ty, augment_r_size)) = rust_type_to_c_type(wrapped_type_ident, r) {
                (quote! { -> #ty }, augment_r_size)
            } else {
                (quote! {}, false)
            }
        }
    }
}
