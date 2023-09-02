use sailfish::TemplateOnce;
use crate::components::component;

#[derive(TemplateOnce, Default)]
#[template(path = "<%= component.path() %>/template.html")]
pub struct <%= component.rust_name() %> {
    value: String,
}

impl <%= component.rust_name() %> {
    pub fn new() -> <%= component.rust_name() %> {
        <%= component.rust_name() %>::default()
    }
}

component!(<%= component.rust_name() %>);
