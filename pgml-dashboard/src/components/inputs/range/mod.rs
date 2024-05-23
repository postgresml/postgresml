use crate::components::stimulus::StimulusTarget as Target;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(Default)]
pub enum InterpolationType {
    #[default]
    Linear,
    Exponential,
}

impl ToString for InterpolationType {
    fn to_string(&self) -> String {
        match self {
            InterpolationType::Linear => String::from("linear"),
            InterpolationType::Exponential => String::from("exponential"),
        }
    }
}

impl From<&str> for InterpolationType {
    fn from(s: &str) -> Self {
        match s {
            "linear" => InterpolationType::Linear,
            "exponential" => InterpolationType::Exponential,
            _ => InterpolationType::Linear,
        }
    }
}

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/range/template.html")]
pub struct Range {
    color: String,
    min: i64,
    max: i64,
    interpolation_type: InterpolationType,
    target: Target,
    initial_value: i64,
}

impl Range {
    pub fn new() -> Range {
        Range {
            color: String::from("slate"),
            min: 1000,
            max: 1000000,
            interpolation_type: InterpolationType::Linear,
            target: Target::new(),
            initial_value: 0,
        }
    }

    pub fn color(mut self, color: &str) -> Self {
        self.color = color.to_string();
        self
    }

    pub fn min(mut self, min: i64) -> Self {
        self.min = min;
        self
    }

    pub fn max(mut self, max: i64) -> Self {
        self.max = max;
        self
    }

    pub fn interpolation_type(mut self, interpolation_type: &str) -> Self {
        self.interpolation_type = InterpolationType::from(interpolation_type);
        self
    }

    pub fn target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    pub fn initial_value(mut self, initial_value: i64) -> Self {
        self.initial_value = initial_value;
        self
    }
}

component!(Range);
