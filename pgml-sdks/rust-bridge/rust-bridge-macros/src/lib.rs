use syn::{parse_macro_input, DeriveInput, ItemImpl};

mod c;
mod common;
mod javascript;
mod python;
mod types;

#[proc_macro_derive(alias)]
pub fn alias(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut output = proc_macro::TokenStream::new();

    let parsed = parse_macro_input!(input as DeriveInput);
    let python_tokens = python::generate_python_alias(parsed.clone());
    let c_tokens = c::generate_c_alias(parsed.clone());
    let javascript_tokens = javascript::generate_javascript_alias(parsed);

    output.extend(python_tokens);
    output.extend(c_tokens);
    output.extend(javascript_tokens);
    output
}

#[proc_macro_attribute]
pub fn alias_methods(
    attributes: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attribute_args = common::AttributeArgs::new(attributes);

    let mut output = input.clone();

    let parsed: ItemImpl = syn::parse(input).unwrap();
    let python_tokens = python::generate_python_methods(parsed.clone(), &attribute_args);
    let c_tokens = c::generate_c_methods(parsed.clone(), &attribute_args);
    let javascript_tokens = javascript::generate_javascript_methods(parsed, &attribute_args);

    output.extend(python_tokens);
    output.extend(c_tokens);
    output.extend(javascript_tokens);
    output
}

#[proc_macro_derive(alias_manual)]
pub fn alias_manual(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut output = proc_macro::TokenStream::new();
    let parsed = parse_macro_input!(input as DeriveInput);
    output.extend(python::generate_alias_manual(parsed.clone()));
    output
}
