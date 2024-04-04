use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "cards/rgb/template.html")]
pub struct Rgb {
    value: Component,
    active: bool,
    link: Option<String>,
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
            active: false,
            link: None,
        }
    }

    pub fn active(mut self) -> Self {
        self.active = true;
        self
    }

    pub fn link(mut self, link: &str) -> Self {
        self.link = Some(link.to_string());
        self
    }
}

component!(Rgb);
