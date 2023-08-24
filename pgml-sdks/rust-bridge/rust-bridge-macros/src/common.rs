use proc_macro2::Ident;
use quote::{format_ident, ToTokens};
use syn::{
    parse::Parser,
    punctuated::Punctuated,
    visit::{self, Visit},
    ImplItemFn, ReturnType, Token, Visibility,
};

use crate::types::{GetOutputType, GetSupportedType, OutputType, SupportedType};

pub struct AttributeArgs {
    pub args: Vec<String>,
}

impl AttributeArgs {
    pub fn new(attributes: proc_macro::TokenStream) -> Self {
        let attribute_parser = Punctuated::<Ident, Token![,]>::parse_terminated;
        let parsed_attributes = attribute_parser
            .parse(attributes)
            .expect("Error parsing attributes for custom_methods macro");
        let args: Vec<String> = parsed_attributes
            .into_pairs()
            .map(|p| p.value().to_string())
            .collect();
        Self { args }
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
