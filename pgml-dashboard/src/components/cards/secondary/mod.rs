use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "cards/secondary/template.html")]
pub struct Secondary {
    value: Component,
}

impl Secondary {
    pub fn new(value: Component) -> Secondary {
        Secondary { value }
    }
}

component!(Secondary);
