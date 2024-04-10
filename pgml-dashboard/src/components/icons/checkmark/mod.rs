use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "icons/checkmark/template.html")]
pub struct Checkmark {
    state: String,
    twitter: bool,
}

impl Checkmark {
    pub fn new() -> Checkmark {
        Checkmark {
            state: String::from("inactive"),
            twitter: false,
        }
    }

    pub fn active(mut self) -> Self {
        self.state = String::from("active");
        self
    }

    pub fn inactive(mut self) -> Self {
        self.state = String::from("inactive");
        self
    }

    pub fn disabled(mut self) -> Self {
        self.state = String::from("disabled");
        self
    }

    pub fn twitter(mut self) -> Self {
        self.twitter = true;
        self
    }
}

component!(Checkmark);
