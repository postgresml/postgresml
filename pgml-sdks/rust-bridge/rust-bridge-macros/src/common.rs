use proc_macro2::{Group, Ident};
use quote::{format_ident, ToTokens};
use syn::{
    parse::{Parse, Parser},
    punctuated::Punctuated,
    token,
    visit::{self, Visit},
    Expr, ExprAssign, ImplItemFn, Lit, ReturnType, Token, Visibility,
};

use crate::types::{GetOutputType, GetSupportedType, OutputType, SupportedType};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SupportedLanguage {
    C,
    Python,
    JavaScript,
}

impl From<&str> for SupportedLanguage {
    fn from(value: &str) -> Self {
        match value {
            "C" => SupportedLanguage::C,
            "Python" => SupportedLanguage::Python,
            "JavaScript" => SupportedLanguage::JavaScript,
            _ => panic!("Cannot convert {value} to SupportedLanguage"),
        }
    }
}

pub struct AttributeArgs {
    args: Vec<Item>,
}

#[derive(Debug, Clone)]
struct Item {
    method: String,
    language_exceptions: Vec<SupportedLanguage>,
}

#[derive(Debug)]
enum AdditionalAttribute {
    Skip(SupportedLanguage),
}

impl From<&ExprAssign> for AdditionalAttribute {
    fn from(value: &ExprAssign) -> Self {
        let a_ty = match &*value.left {
            Expr::Path(p) => p.into_token_stream().to_string(),
            _ => panic!(
                r#"Getting left value - Expected additional attributes to look something like: #[alias_methods(new(skip = "c"))]"#
            ),
        };
        match a_ty.as_str() {
            "skip" => {
                let skip_method = match &*value.right {
                    Expr::Lit(l) => match &l.lit {
                        Lit::Str(l) => l.value().as_str().into(),
                        _ => {
                            panic!(
                                r#"Getting Lit value - Expected additional attributes to look something like: #[alias_methods(new(skip = "c"))]"#
                            )
                        }
                    },
                    _ => panic!(
                        r#"Getting Lit - Expected additional attributes to look something like: #[alias_methods(new(skip = "c"))]"#
                    ),
                };
                AdditionalAttribute::Skip(skip_method)
            }
            _ => panic!("Currently only skip additional attributes are supported"),
        }
    }
}

impl Parse for Item {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let method: Ident = input.parse()?;
        let lookahead = input.lookahead1();
        if !lookahead.peek(token::Paren) {
            Ok(Self {
                method: method.to_string(),
                language_exceptions: Vec::new(),
            })
        } else {
            let group: Group = input.parse()?;
            let group_parser = Punctuated::<ExprAssign, Token![,]>::parse_terminated;
            let parsed_group = group_parser
                .parse(group.stream().into())
                .expect("Error parsing attributes for custom_methods macro");
            let a_atts: Vec<AdditionalAttribute> = parsed_group
                .into_pairs()
                .map(|p| p.value().into())
                .collect();
            // Update this part as needed
            let mut language_exceptions = Vec::new();
            for att in a_atts {
                match att {
                    AdditionalAttribute::Skip(a) => language_exceptions.push(a),
                }
            }
            Ok(Self {
                method: method.to_string(),
                language_exceptions,
            })
        }
    }
}

impl AttributeArgs {
    pub fn new(attributes: proc_macro::TokenStream) -> Self {
        let attribute_parser = Punctuated::<Item, Token![,]>::parse_terminated;
        let parsed_attributes = attribute_parser
            .parse(attributes)
            .expect("Error parsing attributes for custom_methods macro");
        let args: Vec<Item> = parsed_attributes
            .into_pairs()
            .map(|p| p.value().clone())
            .collect();
        Self { args }
    }

    pub fn should_alias_method(&self, method_name: &str, language: SupportedLanguage) -> bool {
        self.args
            .iter()
            .any(|item| item.method == method_name && !item.language_exceptions.contains(&language))
    }
}

#[derive(Debug)]
pub struct GetImplMethod {
    pub exists: bool,
    pub method_ident: Ident,
    pub is_async: bool,
    pub method_arguments: Vec<(String, SupportedType)>,
    pub receiver: Option<proc_macro2::TokenStream>,
    pub output_type: OutputType,
}

impl Default for GetImplMethod {
    fn default() -> Self {
        GetImplMethod {
            exists: false,
            method_ident: format_ident!("nothing"),
            is_async: false,
            method_arguments: Vec::new(),
            receiver: None,
            output_type: OutputType::Default,
        }
    }
}

impl<'ast> Visit<'ast> for GetImplMethod {
    fn visit_impl_item_fn(&mut self, i: &'ast ImplItemFn) {
        if let Visibility::Public(_p) = i.vis {
            self.exists = true;
            visit::visit_impl_item_fn(self, i);
        }
    }

    fn visit_signature(&mut self, i: &'ast syn::Signature) {
        self.method_ident = i.ident.clone();
        self.is_async = i.asyncness.is_some();
        self.output_type = match &i.output {
            ReturnType::Default => OutputType::Default,
            ReturnType::Type(_ra, ty) => {
                let mut get_output_type = GetOutputType::default();
                get_output_type.visit_type(ty);
                get_output_type.output
            }
        };
        visit::visit_signature(self, i);
    }

    fn visit_receiver(&mut self, i: &'ast syn::Receiver) {
        self.receiver = Some(i.to_token_stream());
        visit::visit_receiver(self, i);
    }

    fn visit_pat_type(&mut self, i: &'ast syn::PatType) {
        let pat = i.pat.to_token_stream().to_string();
        let mut ty = GetSupportedType::default();
        ty.visit_type(&i.ty);
        let ty = ty.ty.expect("No type found");
        self.method_arguments.push((pat, ty));
        visit::visit_pat_type(self, i);
    }

    fn visit_expr_return(&mut self, i: &'ast syn::ExprReturn) {
        visit::visit_expr_return(self, i);
    }

    // We don't want to visit any of the statments in the methods
    fn visit_block(&mut self, _i: &'ast syn::Block) {}
}
