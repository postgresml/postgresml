use sailfish::TemplateOnce;

use crate::frontend::components::Component as ComponentModel;

#[derive(TemplateOnce)]
#[template(path = "frontend/templates/component.rs.tpl")]
pub struct Component {
    pub component: ComponentModel,
}

impl Component {
    pub fn new(component: &ComponentModel) -> Self {
        Self {
            component: component.clone(),
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "frontend/templates/template.html.tpl")]
pub struct Html {
    pub component: ComponentModel,
}

impl Html {
    pub fn new(component: &ComponentModel) -> Self {
        Self {
            component: component.clone(),
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

#[derive(TemplateOnce)]
#[template(path = "frontend/templates/sass.scss.tpl")]
pub struct Sass {
    pub component: ComponentModel,
}

impl Sass {
    pub fn new(component: &ComponentModel) -> Self {
        Self {
            component: component.clone(),
        }
    }
}
