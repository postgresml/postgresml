use crate::components::stimulus::stimulus_action::{StimulusAction, StimulusActions};
use pgml_components::component;
use sailfish::TemplateOnce;
use std::path::{Path, PathBuf};

/// Switch button.
#[derive(Clone, Debug)]
pub struct SwitchOption {
    /// Material UI icon.
    pub icon: Option<String>,

    /// SVG icon.
    pub svg: Option<PathBuf>,

    pub value: String,
    pub active: bool,
    pub actions: StimulusActions,
    pub link: Option<String>,
}

impl SwitchOption {
    pub fn new(value: &str) -> Self {
        let mut actions = StimulusActions::default();
        actions.push(
            StimulusAction::new_click()
                .controller("inputs-switch-v-2")
                .method("selectSwitchOption"),
        );

        SwitchOption {
            icon: None,
            svg: None,
            value: value.to_string(),
            active: false,
            actions,
            link: None,
        }
    }

    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn svg(mut self, path: impl AsRef<Path>) -> Self {
        self.svg = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn active(mut self) -> Self {
        self.active = true;
        self
    }

    pub fn set_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn action(mut self, action: StimulusAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn link(mut self, link: impl ToString) -> Self {
        self.link = Some(link.to_string());
        self
    }
}

#[derive(TemplateOnce)]
#[template(path = "inputs/switch_v_2/template.html")]
pub struct SwitchV2 {
    options: Vec<SwitchOption>,
}

impl Default for SwitchV2 {
    fn default() -> Self {
        SwitchV2::new(&[
            SwitchOption::new("CPU").icon("memory"),
            SwitchOption::new("GPU").icon("mode_fan"),
        ])
    }
}

impl SwitchV2 {
    pub fn new(options: &[SwitchOption]) -> SwitchV2 {
        let mut options = options.to_vec();
        let has_active = options.iter().any(|option| option.active);

        if !has_active {
            if let Some(ref mut option) = options.first_mut() {
                option.active = true;
            }
        }

        SwitchV2 { options }
    }
}

component!(SwitchV2);
