#![allow(unused_variables)] // tab.active usage isn't seen inside sailfish templates
use pgml_components::component;
use pgml_components::Component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "navigation/tabs/tab/template.html")]
pub struct Tab {
    content: Component,
    active: bool,
    name: String,
}

impl Tab {
    pub fn new(name: impl ToString, content: Component) -> Tab {
        Tab {
            content,
            active: false,
            name: name.to_string(),
        }
    }

    pub fn button_classes(&self) -> String {
        if self.active {
            "nav-link active btn btn-tertiary rounded-0".to_string()
        } else {
            "nav-link btn btn-tertiary rounded-0".to_string()
        }
    }

    pub fn content_classes(&self) -> String {
        if self.active {
            "tab-pane my-4 show active".to_string()
        } else {
            "tab-pane my-4".to_string()
        }
    }

    pub fn id(&self) -> String {
        format!("tab-{}", self.name.to_lowercase().replace(' ', "-"))
    }

    pub fn selected(&self) -> String {
        if self.active {
            "selected".to_string()
        } else {
            "".to_string()
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn active(mut self) -> Self {
        self.active = true;
        self
    }

    pub fn inactive(mut self) -> Self {
        self.active = false;
        self
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

component!(Tab);
