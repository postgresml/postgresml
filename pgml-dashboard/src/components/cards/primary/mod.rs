use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "cards/primary/template.html")]
pub struct Primary {
    component: Component,
    style: String,
}

impl Primary {
    pub fn new(component: Component) -> Primary {
        Primary {
            component,
            style: "".into(),
        }
    }

    pub fn z_index(mut self, index: i64) -> Self {
        self.style = format!("position: relative; z-index: {};", index);
        self
    }
}

component!(Primary);
