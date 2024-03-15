use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "loading/message/template.html")]
pub struct Message {
    message: String,
}

impl Message {
    pub fn new() -> Message {
        Message {
            message: String::from("Loading..."),
        }
    }

    pub fn message(mut self, message: &str) -> Message {
        self.message = String::from(message);
        self
    }
}

component!(Message);
