use sailfish::TemplateOnce;
use pgml_components::component;

#[derive(TemplateOnce, Default)]
#[template(path = "search/button/template.html")]
pub struct Button {}

impl Button {
    pub fn new() -> Button {
        Button {}
    }
}

component!(Button);
