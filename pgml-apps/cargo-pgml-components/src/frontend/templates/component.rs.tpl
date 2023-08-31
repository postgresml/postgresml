use sailfish::TemplateOnce;
use crate::components::component;

#[derive(TemplateOnce, Default)]
#[template(path = "<%= component_path %>/template.html")]
pub struct <%= component_name %> {
    value: String,
}

impl <%= component_name %> {
    pub fn new() -> <%= component_name %> {
        <%= component_name %>::default()
    }
}

component!(<%= component_name %>);
