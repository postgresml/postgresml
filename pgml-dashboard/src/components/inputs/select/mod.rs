use crate::components::stimulus::stimulus_action::{StimulusAction, StimulusEvents};
use crate::components::stimulus::stimulus_target::StimulusTarget;
use crate::types::CustomOption;
use anyhow::Context;
use pgml_components::component;
use pgml_components::Component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/select/template.html")]
pub struct Select {
    options: Vec<Component>,
    value: String,
    input_value: String,
    offset: String,
    collapsable: bool,
    offset_collapsed: String,
    menu_position: String,
    expandable: bool,
    name: String,
    value_target: StimulusTarget,
    action: CustomOption<StimulusAction>,
}

impl Select {
    pub fn new() -> Select {
        Select {
            options: Vec::new(),
            value: "Select".to_owned(),
            input_value: "Select".to_owned(),
            offset: "0, 10".to_owned(),
            offset_collapsed: "68, -44".to_owned(),
            menu_position: "".to_owned(),
            name: "input_name".to_owned(),
            ..Default::default()
        }
        .options(vec!["option1".to_owned(), "option2".to_owned(), "option3".to_owned()])
    }

    pub fn options<S: ToString>(mut self, values: Vec<S>) -> Self {
        let mut options = Vec::new();
        self.value = values.first().unwrap().to_string();
        self.input_value = values.first().unwrap().to_string();

        for value in values {
            let item = Option::new(
                value.to_string(),
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

    /// Pass in options directly with `value` and `input_value` possibly.
    ///
    /// # Arguments
    ///
    /// * `options` - A list of options to pass in.
    pub fn options_with_input_value(mut self, options: &[self::Option]) -> Self {
        let first_option = options
            .first()
            .with_context(|| "select has no options passed in")
            .unwrap();
        self.value = first_option.value.clone();
        self.input_value = first_option.input_value.clone();

        let mut items = Vec::new();
        for option in options {
            items.push(option.clone().into());
        }
        self.options = items;
        self
    }

    /// Set the value displayed on the dropdown button.
    pub fn value(mut self, value: &str) -> Self {
        self.value = value.to_owned();
        self.input_value = value.to_owned();
        self
    }

    /// The the value of the `<input>` element.
    pub fn input_value(mut self, value: &str) -> Self {
        self.input_value = value.to_owned();
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

    pub fn value_target(mut self, value_target: StimulusTarget) -> Self {
        self.value_target = value_target;
        self
    }

    pub fn action(mut self, action: StimulusAction) -> Self {
        self.action = action.into();
        self
    }
}

#[derive(TemplateOnce, Clone)]
#[template(path = "inputs/select/option.html")]
pub struct Option {
    value: String,
    action: StimulusAction,
    input_value: String,
}

impl Option {
    pub fn new(value: String, action: StimulusAction) -> Self {
        Self {
            value: value.clone(),
            action,
            input_value: value,
        }
    }

    pub fn input_value(mut self, value: String) -> Self {
        self.input_value = value;
        self
    }

    /// Separate the display value of the option from the value passed
    /// into the `<input>` element.
    ///
    /// This is useful when used inside a form. Input values are typically
    /// easily serializable to a backend type, e.g. an integer or a short string,
    /// while the display values are more human-readable.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to display.
    /// * `input_value` - The value to pass into the `<input>` element.
    ///
    pub fn with_input_value(value: impl ToString, input_value: impl ToString) -> Self {
        Self {
            value: value.to_string(),
            input_value: input_value.to_string(),
            action: StimulusAction::new()
                .controller("inputs-select")
                .method("chooseValue")
                .action(StimulusEvents::Click),
        }
    }
}

component!(Option);
component!(Select);
