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
    pub text_target: Option<String>,
    pub range_target: Option<String>,
    pub cost_rate: Option<f32>,
    pub units: String,
}

impl RangeGroup {
    pub fn new(title: &str) -> RangeGroup {
        RangeGroup {
            title: title.to_owned(),
            identifier: title.replace(" ", "_"),
            min: 0,
            max: 100,
            step: 1.0,
            initial_value: 1.0,
            text_target: None,
            range_target: None,
            cost_rate: None,
            units: String::default(),
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

    pub fn text_target(mut self, target: &str) -> Self {
        self.text_target = Some(target.to_owned());
        self
    }

    pub fn range_target(mut self, target: &str) -> Self {
        self.range_target = Some(target.to_owned());
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
}

component!(RangeGroup);
