use proc_macro2::Ident;
use quote::{format_ident, quote, ToTokens};
use std::{
    io::{Read, Write},
    str::FromStr,
};
use syn::{visit::Visit, DeriveInput, ItemImpl, Type};

use crate::{
    common::{AttributeArgs, GetImplMethod},
    types::{OutputType, SupportedType},
};

pub fn generate_c_alias(parsed: DeriveInput) -> proc_macro::TokenStream {
    let name_ident = format_ident!("{}C", parsed.ident);
    let wrapped_type_ident = parsed.ident;
    let wrapped_type_name = wrapped_type_ident.to_string();

    let expanded = quote! {
        #[repr(C)]
        #[cfg(feature = "c")]
        pub struct #name_ident {
            pub wrapped: *mut #wrapped_type_ident
        }

        #[cfg(feature = "c")]
        unsafe impl rust_bridge::c::CustomInto<*mut #name_ident> for #wrapped_type_ident {
            unsafe fn custom_into(self) -> *mut #name_ident {
                Box::into_raw(Box::new(
                    #name_ident {
                        wrapped: Box::into_raw(Box::new(self))
                    }
                ))
            }
        }

        #[cfg(feature = "c")]
        unsafe impl rust_bridge::c::CustomInto<&'static mut #wrapped_type_ident> for *mut #name_ident {
            unsafe fn custom_into(self) -> &'static mut #wrapped_type_ident {
                let c = Box::leak(Box::from_raw(self));
                Box::leak(Box::from_raw(c.wrapped))
            }
        }

        #[cfg(feature = "c")]
        unsafe impl rust_bridge::c::CustomInto<&'static #wrapped_type_ident> for *mut #name_ident {
            unsafe fn custom_into(self) -> &'static #wrapped_type_ident {
                let c = Box::leak(Box::from_raw(self));
                &*Box::leak(Box::from_raw(c.wrapped))
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

        let (
            go_function_arguments,
            go_arguments_prep,
            mut c_function_arguments,
            c_argument_prep,
            rust_function_arguments,
        ) = get_method_arguments(&wrapped_type_ident, &name_ident, &method);

        let method_name = format_ident!("{}_{}", name_ident, method_ident);

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
    proc_macro2::TokenStream,
    proc_macro2::TokenStream,
) {
    let mut go_function_arguments = Vec::new();
    let mut go_arguments_prep = Vec::new();
    let mut c_function_arguments = Vec::new();
    let mut c_argument_prep = Vec::new();
    let mut rust_function_arguments = Vec::new();

    if let Some(_receiver) = &method.receiver {
        c_function_arguments.push(format!("s: *mut {name_ident}"));
        c_argument_prep.push(format!(
            "let s: &mut {wrapped_type_ident} = s.custom_into();"
        ));
        rust_function_arguments.push("s".to_string());
    }

    for (argument_name, argument_type) in &method.method_arguments {
        let (
            go_function_arguments_,
            go_arguments_prep_,
            c_function_arguments_,
            c_argument_prep_,
            rust_function_arguments_,
        ) = get_c_types(argument_name, argument_type);

        go_function_arguments.push(go_function_arguments_);
        go_arguments_prep.push(go_arguments_prep_);
        c_function_arguments.push(c_function_arguments_);
        c_argument_prep.push(c_argument_prep_);
        rust_function_arguments.push(rust_function_arguments_);
    }

    (
        proc_macro2::TokenStream::from_str(&go_function_arguments.join("\n")).unwrap(),
        proc_macro2::TokenStream::from_str(&go_arguments_prep.join("\n")).unwrap(),
        proc_macro2::TokenStream::from_str(&c_function_arguments.join(",")).unwrap(),
        proc_macro2::TokenStream::from_str(&c_argument_prep.join("\n")).unwrap(),
        proc_macro2::TokenStream::from_str(&rust_function_arguments.join(",")).unwrap(),
    )
}

// Need:
// - go function arguments
// - go function argument prep for calling c function
// - go conversion from c returned value - For custom types this is always a wrapper for everything else this is a primitve type
// - c function arguments
// - c function arguments prep for calling rust function
// - arguments to call rust function with
// - c conversion from rust returned value - This is done with the into trait
fn get_c_types(
    argument_name: &str,
    ty: &SupportedType,
) -> (String, String, String, String, String) {
    let t = ty.to_language_string(&None);
    let c_to_rust = format!("let {argument_name}: {t} = {argument_name}.custom_into();");
    match ty {
        SupportedType::Reference(r) => {
            let (
                go_function_arguments,
                go_argument_prep,
                c_function_arguments,
                c_argument_prep,
                rust_function_arguments,
            ) = get_c_types(argument_name, &r.ty);
            (
                "".to_string(),
                "".to_string(),
                c_function_arguments,
                c_to_rust,
                argument_name.to_string(),
            )
        }
        SupportedType::str | SupportedType::String => (
            "".to_string(),
            "".to_string(),
            format!("{argument_name}: *mut std::ffi::c_char"),
            c_to_rust,
            argument_name.to_string(),
        ),
        SupportedType::Option(r) => {
            let (
                go_function_arguments,
                go_argument_prep,
                mut c_function_arguments,
                c_argument_prep,
                rust_function_arguments,
            ) = get_c_types(argument_name, &r);

            (
                "".to_string(),
                "".to_string(),
                c_function_arguments,
                c_to_rust,
                argument_name.to_string(),
            )
        }
        SupportedType::bool => (
            "".to_string(),
            "".to_string(),
            "bool".to_string(),
            "".to_string(),
            argument_name.to_string(),
        ),
        SupportedType::Vec(v) => {
            let (
                go_function_arguments,
                go_argument_prep,
                mut c_function_arguments,
                mut c_argument_prep,
                rust_function_arguments,
            ) = get_c_types(argument_name, v);

            let mut c_function_arguments = c_function_arguments.replacen("*mut", "*mut *mut", 1);
            c_function_arguments.push_str(", v_size: std::ffi::c_ulong");
            c_argument_prep = "let v_size: usize = v_size as usize;".to_string();
            let c_to_rust =
                format!("{c_argument_prep}\nlet {argument_name}: {t} = {argument_name}.custom_into_vec(v_size);");

            (
                "".to_string(),
                "".to_string(),
                c_function_arguments,
                c_to_rust,
                argument_name.to_string(),
            )
        }
        SupportedType::HashMap(_) => panic!("HashMap arguments not supported in c"),
        SupportedType::Tuple(_) => panic!("Tuple arguments not supported in c"),
        SupportedType::S => unreachable!(),
        SupportedType::i64 => (
            "".to_string(),
            "".to_string(),
            format!("{argument_name}: std::ffi::c_longlong"),
            format!("let {argument_name}: {t} = {argument_name} as {t};"),
            argument_name.to_string(),
        ),
        SupportedType::u64 => (
            "".to_string(),
            "".to_string(),
            format!("{argument_name}: std::ffi::c_ulonglong"),
            format!("let {argument_name}: {t} = {argument_name} as {t};"),
            argument_name.to_string(),
        ),
        SupportedType::i32 => (
            "".to_string(),
            "".to_string(),
            format!("{argument_name}: std::ffi::c_long"),
            format!("let {argument_name}: {t} = {argument_name} as {t};"),
            argument_name.to_string(),
        ),
        SupportedType::f64 => (
            "".to_string(),
            "".to_string(),
            format!("{argument_name}: std::ffi::c_double"),
            format!("let {argument_name}: {t} = {argument_name} as {t};"),
            argument_name.to_string(),
        ),
        SupportedType::CustomType(s) => (
            "".to_string(),
            "".to_string(),
            format!("{argument_name}: *mut {s}C"),
            c_to_rust,
            argument_name.to_string(),
        ),
        _ => todo!(),
    }
}

// fn get_c_types(argument_name: &str, ty: &SupportedType) -> (String, Option<String>) {
//     match ty {
//         SupportedType::Reference(r) => get_c_types(&r.ty),
//         SupportedType::str | SupportedType::String => ("*mut std::ffi::c_char".to_string(), None),
//         SupportedType::bool => ("bool".to_string(), None),
//         SupportedType::Vec(v) => {
//             let mut v = get_c_types(v);
//             if !v.0.contains('*') {
//                 v.0 = format!("*mut {}", v.0);
//             }
//             if v.1.is_some() {
//                 panic!("Vec<Vec<_>> not supported in c");
//             }
//             (v.0, Some("std::ffi::c_ulong".to_string()))
//         }
//         SupportedType::HashMap(_) => panic!("HashMap arguments not supported in c"),
//         SupportedType::Option(r) => {
//             let mut t = get_c_types(r);
//             if !t.0.contains('*') {
//                 t.0 = format!("*mut {}", t.0);
//             }
//             t
//         }
//         SupportedType::Tuple(_) => panic!("Tuple arguments not supported in c"),
//         SupportedType::S => unreachable!(),
//         SupportedType::i64 => ("std::ffi::c_longlong".to_string(), None),
//         SupportedType::u64 => ("std::ffi::c_ulonglong".to_string(), None),
//         SupportedType::i32 => ("std::ffi::c_long".to_string(), None),
//         SupportedType::f64 => ("std::ffi::c_double".to_string(), None),
//         SupportedType::CustomType(s) => (format!("*mut {s}"), None),
//     }
// }

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
