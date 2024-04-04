use crate::components::stimulus::stimulus_action::{StimulusAction, StimulusEvents};
use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/text/search/search_option/template.html")]
pub struct SearchOption {
    value: Component,
    action: StimulusAction,
}

impl SearchOption {
    pub fn new(value: Component) -> SearchOption {
        SearchOption {
            value,
            action: StimulusAction::new()
                .controller("inputs-text-search-search")
                .method("selectOption")
                .action(StimulusEvents::FocusIn),
        }
    }
}

component!(SearchOption);
