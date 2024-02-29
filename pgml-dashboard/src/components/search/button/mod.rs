use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "search/button/template.html")]
pub struct Button {}

impl Button {
    pub fn new() -> Button {
        Button {}
    }
}

component!(Button);
