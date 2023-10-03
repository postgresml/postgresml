use crate::components::stimulus::stimulus_action::{StimulusAction, StimulusEvents};
use pgml_components::component;
use pgml_components::Component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/select/template.html")]
pub struct Select {
    options: Vec<Component>,
    value: String,
    offset: String,
    collapsable: bool,
    offset_collapsed: String,
    menu_position: String,
    expandable: bool,
    name: String,
}

impl Select {
    pub fn new() -> Select {
        Select {
            options: Vec::new(),
            value: "Select".to_owned(),
            offset: "0, 10".to_owned(),
            offset_collapsed: "68, -44".to_owned(),
            menu_position: "".to_owned(),
            name: "input_name".to_owned(),
            ..Default::default()
        }
        .options(vec![
            "option1".to_owned(),
            "option2".to_owned(),
            "option3".to_owned(),
        ])
    }

    pub fn options(mut self, values: Vec<String>) -> Self {
        let mut options = Vec::new();
        self.value = values.first().unwrap().to_owned();

        for value in values {
            let item = Option::new(
                value,
                StimulusAction::new()
                    .controller("inputs-select")
                    .method("choose")
                    .action(StimulusEvents::Click),
            );
            options.push(item.into());
        }

        self.options = options;
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    pub fn text(mut self, value: String) -> Self {
        self.value = value;
        self
    }

    pub fn collapsable(mut self) -> Self {
        self.collapsable = true;
        self
    }

    pub fn menu_end(mut self) -> Self {
        self.menu_position = "dropdown-menu-end".to_owned();
        self
    }

    pub fn menu_start(mut self) -> Self {
        self.menu_position = "dropdown-menu-start".to_owned();
        self
    }

    pub fn offset(mut self, offset: &str) -> Self {
        self.offset = offset.to_owned();
        self
    }

    pub fn offset_collapsed(mut self, offset: &str) -> Self {
        self.offset_collapsed = offset.to_owned();
        self
    }

    pub fn expandable(mut self) -> Self {
        self.expandable = true;
        self
    }
}

#[derive(TemplateOnce)]
#[template(path = "inputs/select/option.html")]
pub struct Option {
    value: String,
    action: StimulusAction,
}

impl Option {
    pub fn new(value: String, action: StimulusAction) -> Self {
        Option { value, action }
    }
}

component!(Option);
component!(Select);
