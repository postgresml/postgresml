use crate::components::stimulus::stimulus_target::StimulusTarget;
use pgml_components::component;
use sailfish::TemplateOnce;
use std::fmt;

pub enum Headers {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl fmt::Display for Headers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Headers::H1 => write!(f, "h1"),
            Headers::H2 => write!(f, "h2"),
            Headers::H3 => write!(f, "h3"),
            Headers::H4 => write!(f, "h4"),
            Headers::H5 => write!(f, "h5"),
            Headers::H6 => write!(f, "h6"),
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "inputs/text/editable_header/template.html")]
pub struct EditableHeader {
    value: String,
    header_type: Headers,
    input_target: StimulusTarget,
    input_name: Option<String>,
    id: String,
}

impl Default for EditableHeader {
    fn default() -> Self {
        Self {
            value: String::from("Title Goes Here"),
            header_type: Headers::H3,
            input_target: StimulusTarget::new(),
            input_name: None,
            id: String::from(""),
        }
    }
}

impl EditableHeader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn header_type(mut self, header_type: Headers) -> Self {
        self.header_type = header_type;
        self
    }

    pub fn value(mut self, value: &str) -> Self {
        self.value = value.to_string();
        self
    }

    pub fn input_target(mut self, input_target: StimulusTarget) -> Self {
        self.input_target = input_target;
        self
    }

    pub fn input_name(mut self, input_name: &str) -> Self {
        self.input_name = Some(input_name.to_string());
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }
}

component!(EditableHeader);
