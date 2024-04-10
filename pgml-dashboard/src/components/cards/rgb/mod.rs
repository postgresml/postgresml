use pgml_components::{component, Component};
use sailfish::TemplateOnce;

use crate::components::stimulus::StimulusAction;
use crate::types::CustomOption;

#[derive(TemplateOnce)]
#[template(path = "cards/rgb/template.html")]
pub struct Rgb {
    value: Component,
    link: Option<String>,
    link_action: CustomOption<StimulusAction>,
    controller_classes: Vec<String>,
    card_classes: Vec<String>,
    body_classes: Vec<String>,
}

impl Default for Rgb {
    fn default() -> Self {
        Rgb::new("RGB card".into())
    }
}

impl Rgb {
    pub fn new(value: Component) -> Rgb {
        Rgb {
            value,
            link: None,
            link_action: CustomOption::default(),
            controller_classes: vec![],
            card_classes: vec![],
            body_classes: vec![],
        }
    }

    pub fn active(mut self) -> Self {
        self.card_classes.push("active".into());
        self.card_classes.push("main-gradient-border-card-1".into());
        self
    }

    pub fn is_active(mut self, active: bool) -> Self {
        if active {
            self.card_classes.push("active".into());
            self.card_classes.push("main-gradient-border-card-1".into());
        }

        self
    }

    pub fn link(mut self, link: &str) -> Self {
        self.link = Some(link.to_string());
        self
    }

    pub fn link_action(mut self, action: StimulusAction) -> Self {
        self.link_action = action.into();
        self
    }

    pub fn h_100(mut self) -> Self {
        self.controller_classes.push("h-100".into());
        self.card_classes.push("h-100".into());
        self
    }
}

component!(Rgb);
