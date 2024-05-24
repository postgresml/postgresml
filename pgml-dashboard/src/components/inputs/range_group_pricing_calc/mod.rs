use crate::components::inputs::range::InterpolationType;
use crate::components::stimulus::StimulusTarget;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/range_group_pricing_calc/template.html")]
pub struct RangeGroupPricingCalc {
    interpolation_type: InterpolationType,
    include_slider: bool,
    min: i64,
    max: i64,
    target: StimulusTarget,
    label: String,
    name: String,
    initial_value: i64,
}

impl RangeGroupPricingCalc {
    pub fn new() -> RangeGroupPricingCalc {
        RangeGroupPricingCalc {
            interpolation_type: InterpolationType::Linear,
            include_slider: true,
            min: 0,
            max: 1000000,
            target: StimulusTarget::new(),
            label: String::from(""),
            name: String::from(""),
            initial_value: 0,
        }
    }

    pub fn interpolation_type(mut self, interpolation_type: &str) -> Self {
        self.interpolation_type = InterpolationType::from(interpolation_type);
        self
    }

    pub fn include_slider(mut self, include_slider: bool) -> Self {
        self.include_slider = include_slider;
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

    pub fn target(mut self, target: StimulusTarget) -> Self {
        self.target = target;
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn initial_value(mut self, initial_value: i64) -> Self {
        self.initial_value = initial_value;
        self
    }
}

component!(RangeGroupPricingCalc);
