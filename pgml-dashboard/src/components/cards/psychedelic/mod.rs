use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "cards/psychedelic/template.html")]
pub struct Psychedelic {
    border_only: bool,
    color: String,
    content: Component,
}

impl Psychedelic {
    pub fn new() -> Psychedelic {
        Psychedelic {
            border_only: false,
            color: String::from("blue"),
            content: Component::default(),
        }
    }

    pub fn is_border_only(mut self, border_only: bool) -> Self {
        self.border_only = border_only;
        self
    }

    pub fn set_color_pink(mut self) -> Self {
        self.color = String::from("pink");
        self
    }

    pub fn set_color_blue(mut self) -> Self {
        self.color = String::from("green");
        self
    }

    pub fn set_content(mut self, content: Component) -> Self {
        self.content = content;
        self
    }
}

component!(Psychedelic);
