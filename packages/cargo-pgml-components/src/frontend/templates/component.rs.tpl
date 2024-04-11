use sailfish::TemplateOnce;
use pgml_components::component;

#[derive(TemplateOnce, Default)]
#[template(path = "<%= component.path() %>/template.html")]
pub struct <%= component.rust_name() %> {}

impl <%= component.rust_name() %> {
    pub fn new() -> <%= component.rust_name() %> {
        <%= component.rust_name() %> {}
    }
}

component!(<%= component.rust_name() %>);
