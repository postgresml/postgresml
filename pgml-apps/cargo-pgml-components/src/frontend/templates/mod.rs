use sailfish::TemplateOnce;

use crate::frontend::components::Component as ComponentModel;

#[derive(TemplateOnce)]
#[template(path = "frontend/templates/component.rs.tpl")]
pub struct Component {
    pub component_name: String,
    pub component_path: String,
}

impl Component {
    pub fn new(component: &ComponentModel) -> Self {
        Self {
            component_name: component.name(),
            component_path: component.path(),
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "frontend/templates/template.html.tpl")]
pub struct Html {
    pub controller_name: String,
}

impl Html {
    pub fn new(component: &ComponentModel) -> Self {
        Self {
            controller_name: component.path().replace("_", "-"),
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "frontend/templates/stimulus.js.tpl")]
pub struct Stimulus {
    pub controller_name: String,
}

impl Stimulus {
    pub fn new(component: &ComponentModel) -> Self {
        Self {
            controller_name: component.controller_name(),
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "frontend/templates/mod.rs.tpl")]
pub struct Mod {
    pub modules: Vec<ComponentModel>,
}
