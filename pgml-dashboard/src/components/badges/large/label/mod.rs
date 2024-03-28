use crate::components::stimulus::StimulusAction;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(Clone, Debug)]
pub struct LabelCloseOptions {
    pub action: StimulusAction,
    pub url: String,
}

#[derive(TemplateOnce, Default)]
#[template(path = "badges/large/label/template.html")]
pub struct Label {
    value: String,
    close_options: Option<LabelCloseOptions>,
    active: String,
}

impl Label {
    pub fn new(value: &str) -> Label {
        Label {
            value: value.into(),
            close_options: None,
            active: "".into(),
        }
    }

    pub fn close_options(mut self, options: LabelCloseOptions) -> Label {
        self.close_options = Some(options);
        self
    }

    pub fn active(mut self) -> Label {
        self.active = "active".into();
        self
    }
}

component!(Label);
