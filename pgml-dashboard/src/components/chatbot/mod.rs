use sailfish::TemplateOnce;
use pgml_components::component;

#[derive(TemplateOnce, Default)]
#[template(path = "chatbot/template.html")]
pub struct Chatbot {
    value: String,
}

impl Chatbot {
    pub fn new() -> Chatbot {
        Chatbot {
            value: String::from("src/components/chatbot"),
        }
    }
}

component!(Chatbot);