use sailfish::TemplateOnce;
use pgml_components::component;

#[derive(TemplateOnce, Default)]
#[template(path = "icons/twitter/template.html")]
pub struct Twitter {}

impl Twitter {
    pub fn new() -> Twitter {
        Twitter {}
    }
}

component!(Twitter);
