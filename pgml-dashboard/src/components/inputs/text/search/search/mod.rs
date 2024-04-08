use pgml_components::component;
use sailfish::TemplateOnce;

use crate::components::inputs::text::Input;
use crate::components::stimulus::stimulus_action::{StimulusAction, StimulusEvents};

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub name: String,
    pub placeholder: String,
    pub search_url: String,
    pub id: String,
}

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/text/search/search/template.html")]
pub struct Search {
    input: Input,
    search_url: String,
    id: String,
}

impl Search {
    pub fn new(options: SearchOptions) -> Search {
        Search {
            input: Input::new()
                .label(options.name.into())
                .icon("search")
                .placeholder(options.placeholder)
                .input_action(
                    StimulusAction::new()
                        .controller("inputs-text-search-search")
                        .method("startSearch")
                        .action(StimulusEvents::FocusIn),
                )
                .input_action(
                    StimulusAction::new()
                        .controller("inputs-text-search-search")
                        .method("searchDebounced")
                        .action(StimulusEvents::KeyUp),
                ),
            search_url: options.search_url,
            id: options.id,
        }
    }

    pub fn get_input(&self) -> Input {
        self.input.clone()
    }

    pub fn with_input(mut self, input: Input) -> Self {
        self.input = input;
        self
    }

    /// Close the dropdown whenever you want.
    /// Modify the action to change the event from the default onClick.
    pub fn end_search_action() -> StimulusAction {
        StimulusAction::new_click()
            .controller("inputs-text-search-search")
            .method("endSearch")
    }
}

component!(Search);
