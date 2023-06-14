use quote::{format_ident, quote, ToTokens};
use syn::{visit::Visit, DeriveInput, Ident, ItemImpl, Type};

use crate::common::{AttributeArgs, GetImplMethod, OutputType};
use crate::types::SupportedType;

pub fn generate_javascript_derive(parsed: DeriveInput) -> proc_macro::TokenStream {
    let name = format_ident!("{}Javascript", parsed.ident);
    let wrapped_type = format_ident!("{}", parsed.ident);

    let expanded = quote! {
        pub struct #name {
            wrapped: #wrapped_type
        }

        impl Finalize for #name {}

        impl From<#wrapped_type> for #name {
            fn from(w: #wrapped_type) -> Self {
                Self {
                    wrapped: w,
                }
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

    let wrapped_type = match *parsed.self_ty {
        Type::Path(p) => p.path.segments.first().unwrap().ident.clone(),
        _ => panic!("How did you even cause this error to happen"),
    };
    let name = format_ident!("{}Javascript", wrapped_type);

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

        let method_ident = format_ident!("{}", method.method_ident);

        let (method_arguments, wrapper_arguments) =
            get_method_wrapper_arguments_javascript(&method);
        
        let (output_type, convert_from) = match &method.output_type {
            OutputType::Result(v) | OutputType::Other(v) => {
                convert_output_type_convert_from_javascript(v, &method)
            }
            OutputType::Default => (None, None),
        };
        
        let method_name_string = format!("{}", method_ident.to_string());
        object_sets.push(quote! {
            let f: Handle<JsFunction> = JsFunction::new(cx, #name::#method_ident)?;
            obj.set(cx, #method_name_string, f)?;
        });
        
        let signature = quote! {
            pub fn #method_ident<'a>(mut cx: FunctionContext<'a>) -> #output_type
        };
        let middle = quote! {
            #(#method_arguments)*
        };
        
        let mq = if method_ident.to_string() == "new" {
            if method.is_async {
                quote! {
                    #signature {
                        #middle
                        let channel = cx.channel();
                        let (deferred, promise) = cx.promise();
                        deferred.try_settle_with(&channel, move |mut cx| {
                            let runtime = get_or_set_runtime();
                            let x = runtime.block_on(#wrapped_type::new(#(#wrapper_arguments),*)).unwrap();
                            let x = #name::from(x);
                            x.to_object(&mut cx)
                        }).expect("Error sending js");
                        Ok(promise)
                    }
                }
            } else {
                panic!("We do not support non async javascript constructors yet")
            }
        } else {
            let (middle, call) = if let Some(_r) = &method.receiver {
                (
                    quote! {
                        #middle
                        let this = cx.this();
                        let s: Handle<JsBox<RefCell<#name>>> = this.get(&mut cx, "s")?;
                        let wrapped = s.borrow().wrapped.clone();
                    },
                    quote! {
                        wrapped.#method_ident(#(#wrapper_arguments),*)
                    },
                )
            } else {
                (
                    middle,
                    quote! {
                        #wrapped_type::#method_ident(#(#wrapper_arguments),*)
                    },
                )
            };
            let call = if method.is_async {
                quote! {
                    runtime.block_on(#call)
                }
            } else {
                call
            };
            let call = if let OutputType::Result(_r) = &method.output_type {
                quote! {
                    #call.expect("Need better rust error handeling in javascript")
                }
            } else {
                call
            };
            let call = if let Some(convert) = convert_from {
                quote! {
                    let x = #convert::from(#call);
                    x.to_object(&mut cx)
                }
            } else {
                match &method.output_type {
                    OutputType::Other(v) | OutputType::Result(v) => {
                        let cx_to_function = get_javascript_cx_to_function(&v);
                        match cx_to_function {
                            Some(c) => quote! { Ok(#c(#call)) },
                            None => quote! {
                                #call;
                                Ok(JsUndefined::new(&mut cx))
                            },
                        }
                    }
                    OutputType::Default => quote! {
                        #call;
                    },
                }
            };
            if method.is_async {
                quote! {
                    #signature {
                        #middle
                        let channel = cx.channel();
                        let (deferred, promise) = cx.promise();
                        deferred.try_settle_with(&channel, move |mut cx| {
                            let runtime = get_or_set_runtime();
                            #call
                        }).expect("Error sending js");
                        Ok(promise)
                    }
                }
            } else {
                quote! {
                    #signature {
                        #middle
                        #call
                    }
                }
            }
        };
        methods.push(mq)
    }

    proc_macro::TokenStream::from(quote! {
        impl #name {
            #(#methods)*

            pub fn to_object<'a>(self, cx: &mut impl Context<'a>) -> JsResult<'a, JsObject> {
                let obj = cx.empty_object();
                let s = cx.boxed(RefCell::new(self));
                obj.set(cx, "s", s)?;
                #(#object_sets)*
                Ok(obj)
            }
        }
    })
}

fn handle_jsarray_to_vec(
    argument_name_ident: &Ident,
    ty: &SupportedType,
) -> proc_macro2::TokenStream {
    let ty_ty = ty.to_type().expect("Error parsing type in handle_jsarray_to_vec");
    let closure = build_closure_from_js_to_rust(ty);

    quote! {
        let #argument_name_ident = #argument_name_ident.to_vec(&mut cx)?;
        let #argument_name_ident = #argument_name_ident.into_iter().map(#closure).collect::<Vec<#ty_ty>>();
    }
}

fn build_closure_from_js_to_rust(ty: &SupportedType) -> proc_macro2::TokenStream {
    match ty {
        SupportedType::HashMap((k, v)) => {
            quote! {
                |jt| {
                    HashMap::new()
                }
            }
        }
        _ => quote! {},
    }
}

pub fn get_method_wrapper_arguments_javascript(
    method: &GetImplMethod,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    let mut method_arguments = Vec::new();
    let mut wrapper_arguments = Vec::new();

    method
        .method_arguments
        .iter()
        .enumerate()
        .for_each(|(i, (argument_name, argument_type))| {
            let argument_name_ident = format_ident!("{}", argument_name);
            match argument_type {
                SupportedType::Reference(r) => match **r {
                    SupportedType::str => {
                        method_arguments.push(quote! {
                            let #argument_name_ident: Handle<JsString> = cx.argument(#i as i32)?;
                            let #argument_name_ident = #argument_name_ident.value(&mut cx);
                        });
                        wrapper_arguments.push(quote! {
                            & #argument_name_ident
                        });
                    },
                    _ => panic!(
                        "Javscript get_method_wrapper_arguments_javascript not implimented for type: {}",
                        argument_type.to_string()
                    ),
                },
                SupportedType::Vec(v) => {
                    let vec_conversion = handle_jsarray_to_vec(&argument_name_ident, v);
                    method_arguments.push(quote! {
                        let #argument_name_ident: Handle<JsArray> = cx.argument(#i as i32)?;
                        #vec_conversion
                    });
                    wrapper_arguments.push(quote! {
                        #argument_name_ident
                    });
                },
                _ => {
                    // let ty = argument_type
                    //     .to_type()
                    //     .expect("Error parsing type in get_method_wrapper_arguments_javascript");
                    // method_arguments.push(quote! {
                    //     #argument_name_ident : #ty
                    // });
                    // wrapper_arguments.push(quote! {
                    //     #argument_name_ident
                    // });
                }
            };
        });

    (method_arguments, wrapper_arguments)
}

pub fn convert_output_type_convert_from_javascript(
    ty: &str,
    method: &GetImplMethod,
) -> (
    Option<proc_macro2::TokenStream>,
    Option<proc_macro2::TokenStream>,
) {
    let (output_type, convert_from) = match ty {
        "Self" => (
            Some(quote! {JsResult<'a, JsObject>}),
            Some(format_ident!("{}", method.method_ident).into_token_stream()),
        ),
        "String" => (Some(quote! {JsResult<'a, JsString>}), None),
        "()" => (None, None),
        // TODO: Add handeling for other javascript types here
        o @ _ => (
            Some(quote! {JsResult<'a, JsObject>}),
            Some(format_ident!("{}Javascript", o).into_token_stream()),
        ),
    };

    if method.is_async {
        (Some(quote! {JsResult<'a, JsPromise>}), convert_from)
    } else {
        (output_type, convert_from)
    }
}

fn get_javascript_cx_to_function(output: &str) -> Option<proc_macro2::TokenStream> {
    match output {
        "String" => Some(quote! {cx.string}),
        "()" => None,
        _ => panic!("Cx type not yet implimented"),
    }
}
