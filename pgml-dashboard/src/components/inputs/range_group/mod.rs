use crate::components::stimulus::stimulus_target::StimulusTarget;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/range_group/template.html")]
pub struct RangeGroup {
    pub title: String,
    pub identifier: String,
    pub min: i64,
    pub max: i64,
    pub step: f32,
    pub initial_value: f64,
    pub text_target: StimulusTarget,
    pub range_target: StimulusTarget,
    pub cost_rate: Option<f32>,
    pub units: String,
    pub group_target: StimulusTarget,
}

impl RangeGroup {
    pub fn new(title: &str) -> RangeGroup {
        RangeGroup {
            title: title.to_owned(),
            identifier: title.replace(" ", "_").to_lowercase(),
            min: 0,
            max: 100,
            step: 1.0,
            initial_value: 1.0,
            text_target: StimulusTarget::new(),
            range_target: StimulusTarget::new(),
            cost_rate: None,
            units: String::default(),
            group_target: StimulusTarget::new(),
        }
    }

    pub fn identifier(mut self, identifier: &str) -> Self {
        self.identifier = identifier.replace(" ", "_").to_owned();
        self
    }

    pub fn bounds(mut self, min: i64, max: i64, step: f32) -> Self {
        self.min = min;
        self.max = max;
        self.step = step;
        self
    }

    pub fn initial_value(mut self, value: f64) -> Self {
        self.initial_value = value;
        self
    }

    pub fn text_target(mut self, target: StimulusTarget) -> Self {
        self.text_target = target;
        self
    }

    pub fn range_target(mut self, target: StimulusTarget) -> Self {
        self.range_target = target.to_owned();
        self
    }

    pub fn cost_rate(mut self, cost_rate: f32) -> Self {
        self.cost_rate = Some(cost_rate);
        self
    }

    pub fn units(mut self, units: &str) -> Self {
        self.units = units.to_owned();
        self
    }

    pub fn group_target(mut self, target: StimulusTarget) -> Self {
        self.group_target = target;
        self
    }
}

component!(RangeGroup);
