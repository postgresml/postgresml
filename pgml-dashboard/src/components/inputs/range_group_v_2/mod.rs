use pgml_components::component;
use sailfish::TemplateOnce;

use crate::components::stimulus::{stimulus_action::StimulusActions, StimulusAction};
use std::collections::BTreeSet;

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/range_group_v_2/template.html")]
pub struct RangeGroupV2 {
    name: String,
    min: String,
    max: String,
    step: String,
    value: String,
    unit: String,
    input_unit: String,
    input_classes: BTreeSet<String>,
    cost_per_unit: String,
    cost_frequency: String,

    actions: StimulusActions,
}

impl RangeGroupV2 {
    pub fn new() -> RangeGroupV2 {
        Self {
            input_classes: BTreeSet::from_iter(vec!["form-control".to_string()].into_iter()),
            ..Default::default()
        }
        .min("40")
        .max("16000")
        .unit("GB")
        .cost_per_unit("0.20")
        .value("40")
        .cost_frequency("h")
    }

    pub fn name(mut self, name: impl ToString) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn min(mut self, min: impl ToString) -> Self {
        self.min = min.to_string();
        self
    }

    pub fn max(mut self, max: impl ToString) -> Self {
        self.max = max.to_string();
        self
    }

    pub fn step(mut self, step: impl ToString) -> Self {
        self.step = step.to_string();
        self
    }

    pub fn value(mut self, value: impl ToString) -> Self {
        self.value = value.to_string();
        self
    }

    pub fn unit(mut self, unit: impl ToString) -> Self {
        self.unit = unit.to_string();
        self.input_unit = unit.to_string();

        self.with_input_classes()
    }

    pub fn input_unit(mut self, input_unit: impl ToString) -> Self {
        self.input_unit = input_unit.to_string();
        self.with_input_classes()
    }

    pub fn cost_per_unit(mut self, cost_per_unit: impl ToString) -> Self {
        self.cost_per_unit = cost_per_unit.to_string();
        self
    }

    pub fn cost_frequency(mut self, cost_frequency: impl ToString) -> Self {
        self.cost_frequency = cost_frequency.to_string();
        self
    }

    pub fn action(mut self, action: StimulusAction) -> Self {
        self.actions.push(action);
        self
    }

    fn with_input_classes(mut self) -> Self {
        if !self.input_unit.is_empty() {
            self.input_classes
                .insert("inputs-range-group-v-2-with-unit".to_string());
        } else {
            self.input_classes.remove("inputs-range-group-v-2-with-unit");
        }

        self
    }
}

component!(RangeGroupV2);
