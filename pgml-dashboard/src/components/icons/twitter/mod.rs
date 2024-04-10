use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "icons/twitter/template.html")]
pub struct Twitter {}

impl Twitter {
    pub fn new() -> Twitter {
        Twitter {}
    }
}

component!(Twitter);
