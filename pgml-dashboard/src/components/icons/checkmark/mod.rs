use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "icons/checkmark/template.html")]
pub struct Checkmark {
    color: String,
    twitter: bool,
    disabled: bool,
}

impl Checkmark {
    pub fn new() -> Checkmark {
        Checkmark {
            color: String::from("blue"),
            twitter: false,
            disabled: false,
        }
    }

    pub fn color(mut self, color: &str) -> Self {
        self.color = String::from(color);
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    pub fn twitter(mut self) -> Self {
        self.twitter = true;
        self
    }
}

component!(Checkmark);
