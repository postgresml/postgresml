use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "cards/rgb/template.html")]
pub struct Rgb {
    value: Component,
    link: Option<String>,
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

    pub fn link(mut self, link: &str) -> Self {
        self.link = Some(link.to_string());
        self
    }

    pub fn h_100(mut self) -> Self {
        self.controller_classes.push("h-100".into());
        self.card_classes.push("h-100".into());
        self
    }
}

component!(Rgb);
