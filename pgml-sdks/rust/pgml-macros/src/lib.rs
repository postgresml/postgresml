use syn::{parse_macro_input, DeriveInput, ItemImpl};

mod common;
mod python;
mod types;

#[proc_macro_derive(custom_derive)]
pub fn custom_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut output = proc_macro::TokenStream::new();

    let parsed = parse_macro_input!(input as DeriveInput);
    let python_tokens = python::generate_python_derive(parsed);

    output.extend(python_tokens);
    output
}

#[proc_macro_attribute]
pub fn custom_methods(
    attributes: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attribute_args = common::AttributeArgs::new(attributes);

    let mut output = input.clone();

    let parsed: ItemImpl = syn::parse(input).unwrap();
    let python_tokens = python::generate_python_methods(parsed, &attribute_args);

    output.extend(python_tokens);
    output
}

#[proc_macro_derive(custom_into_py)]
pub fn custom_into_py(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    python::generate_into_py(parsed)
}
