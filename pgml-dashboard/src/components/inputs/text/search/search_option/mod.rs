use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/text/search/search_option/template.html")]
pub struct SearchOption {
    value: Component,
}

impl SearchOption {
    pub fn new(value: Component) -> SearchOption {
        SearchOption { value }
    }
}

component!(SearchOption);
