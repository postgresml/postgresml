use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "cards/primary/template.html")]
pub struct Primary {
    component: Component,
}

impl Primary {
    pub fn new(component: Component) -> Primary {
        Primary { component }
    }
}

component!(Primary);
