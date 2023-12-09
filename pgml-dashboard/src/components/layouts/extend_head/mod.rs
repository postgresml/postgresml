use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "layouts/extend_head/template.html")]
pub struct ExtendHead {}

impl ExtendHead {
    pub fn new() -> ExtendHead {
        ExtendHead {}
    }
}

component!(ExtendHead);
