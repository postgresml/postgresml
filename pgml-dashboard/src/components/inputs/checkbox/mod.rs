use pgml_components::{component, Component};
use sailfish::TemplateOnce;

use crate::utils::random_string;

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/checkbox/template.html")]
pub struct Checkbox {
    name: String,
    value: String,
    label: Component,
    id: String,
}

impl Checkbox {
    pub fn new(name: &str, value: &str) -> Checkbox {
        Checkbox {
            name: name.to_string(),
            value: value.to_string(),
            label: Component::from(name),
            id: random_string(16).to_lowercase(),
        }
    }
}

component!(Checkbox);
