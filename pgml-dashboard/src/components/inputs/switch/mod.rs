use crate::components::stimulus::stimulus_action::StimulusAction;
use crate::components::stimulus::stimulus_target::StimulusTarget;
use pgml_components::component;
use sailfish::TemplateOnce;
use std::fmt::{self, Display, Formatter};

pub enum State {
    Left,
    Right,
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            State::Left => write!(f, "left"),
            State::Right => write!(f, "right"),
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "inputs/switch/template.html")]
pub struct Switch {
    left_value: String,
    left_icon: String,
    right_value: String,
    right_icon: String,
    initial_state: State,
    on_toggle: Vec<StimulusAction>,
    target: StimulusTarget,
}

impl Default for Switch {
    fn default() -> Self {
        Switch {
            left_value: String::from("left"),
            left_icon: String::from(""),
            right_value: String::from("right"),
            right_icon: String::from(""),
            on_toggle: Vec::new(),
            initial_state: State::Left,
            target: StimulusTarget::new(),
        }
    }
}

impl Switch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn left(mut self, value: &str, icon: &str) -> Switch {
        self.left_value = value.into();
        self.left_icon = icon.into();
        self
    }

    pub fn right(mut self, value: &str, icon: &str) -> Switch {
        self.right_value = value.into();
        self.right_icon = icon.into();
        self
    }

    pub fn on_toggle(mut self, action: StimulusAction) -> Switch {
        self.on_toggle.push(action);
        self
    }

    pub fn default_position(mut self, state: State) -> Switch {
        self.initial_state = state;
        self
    }

    pub fn target(mut self, target: StimulusTarget) -> Switch {
        self.target = target;
        self
    }
}

component!(Switch);
