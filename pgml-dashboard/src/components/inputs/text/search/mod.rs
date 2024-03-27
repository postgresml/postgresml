use pgml_components::component;
use sailfish::TemplateOnce;

use crate::components::inputs::text::Input;
use crate::components::stimulus::StimulusAction;

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/text/search/template.html")]
pub struct Search {
    input: Input,
}

impl Search {
    pub fn new() -> Search {
        Search { input: Input::new() }
    }

    pub fn get_input(&self) -> Input {
        self.input.clone()
    }

    pub fn with_input(mut self, input: Input) -> Self {
        self.input = input;
        self
    }
}

component!(Search);
