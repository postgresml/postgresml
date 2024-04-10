use crate::components::stimulus::stimulus_action::{StimulusAction, StimulusActions};
use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "inputs/text/input/template.html")]
pub struct Input {
    label: Option<Component>,
    name: String,
    type_: String,
    icon: Option<String>,
    id: String,
    placeholder: String,
    icon_actions: StimulusActions,
    input_actions: StimulusActions,
    autocomplete: bool,
    value: String,
    required: bool,
    error: Option<String>,
}

impl Input {
    pub fn new() -> Input {
        let mut icon_actions = StimulusActions::default();
        icon_actions.push(
            StimulusAction::new_click()
                .controller("inputs-text-input")
                .method("clickIcon"),
        );
        Input {
            id: crate::utils::random_string(16),
            label: None,
            name: "".into(),
            type_: "text".into(),
            icon: None,
            placeholder: "".into(),
            icon_actions,
            input_actions: StimulusActions::default(),
            autocomplete: false,
            value: "".to_string(),
            required: false,
            error: None,
        }
    }

    pub fn icon(mut self, icon: impl ToString) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn label(mut self, label: Component) -> Self {
        self.label = Some(label);
        self
    }

    pub fn placeholder(mut self, placeholder: impl ToString) -> Self {
        self.placeholder = placeholder.to_string();
        self
    }

    pub fn id(mut self, id: impl ToString) -> Self {
        self.id = id.to_string();
        self
    }

    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn type_(mut self, type_: impl ToString) -> Self {
        self.type_ = type_.to_string();
        self
    }

    pub fn icon_action(mut self, action: StimulusAction) -> Self {
        self.icon_actions.push(action);
        self
    }

    pub fn input_action(mut self, action: StimulusAction) -> Self {
        self.input_actions.push(action);
        self
    }

    pub fn value(mut self, value: impl ToString) -> Self {
        self.value = value.to_string();
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn error(mut self, error: Option<impl ToString>) -> Self {
        self.error = error.map(|e| e.to_string());
        self
    }
}

component!(Input);
