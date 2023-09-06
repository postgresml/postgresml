use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "confirm_modal/template.html")]
pub struct ConfirmModal {
    confirm_question: String,
    confirm_text: String,
    confirm_action: String,
    decline_text: String,
    decline_action: String,
}

impl ConfirmModal {
    pub fn new(confirm_question: &str) -> ConfirmModal {
        ConfirmModal {
            confirm_question: confirm_question.to_owned(),
            confirm_text: "Yes".to_owned(),
            confirm_action: "".to_owned(),
            decline_text: "No".to_owned(),
            decline_action: "".to_owned(),
        }
    }

    pub fn confirm_action(mut self, confirm_action: &str) -> ConfirmModal {
        self.confirm_action = confirm_action.to_owned();
        self
    }
}

component!(ConfirmModal);
