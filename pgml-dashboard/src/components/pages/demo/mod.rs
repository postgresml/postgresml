use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "pages/demo/template.html")]
pub struct Demo {}

impl Demo {
    pub fn new() -> Demo {
        Demo {}
    }
}

component!(Demo);
