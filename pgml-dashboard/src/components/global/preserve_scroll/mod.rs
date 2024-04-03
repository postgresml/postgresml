use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "global/preserve_scroll/template.html")]
pub struct PreserveScroll {}

impl PreserveScroll {
    pub fn new() -> PreserveScroll {
        PreserveScroll {}
    }
}

component!(PreserveScroll);
