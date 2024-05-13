use sailfish::runtime::{Buffer, Render};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum StimulusEvents {
    Click,
    Change,
    Submit,
    Input,
    Toggle,
    FocusOut,
    FocusIn,
    KeyDown,
    KeyUp,
    KeyDownWithKey(String),
}

impl fmt::Display for StimulusEvents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            StimulusEvents::Click => write!(f, "click"),
            StimulusEvents::Change => write!(f, "change"),
            StimulusEvents::Submit => write!(f, "submit"),
            StimulusEvents::Input => write!(f, "input"),
            StimulusEvents::Toggle => write!(f, "toggle"),
            StimulusEvents::FocusOut => write!(f, "focusout"),
            StimulusEvents::FocusIn => write!(f, "focusin"),
            StimulusEvents::KeyDown => write!(f, "keydown"),
            StimulusEvents::KeyUp => write!(f, "keyup"),
            StimulusEvents::KeyDownWithKey(ref key) => write!(f, "keydown.{}", key),
        }
    }
}

impl FromStr for StimulusEvents {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, ()> {
        match input {
            "click" => Ok(StimulusEvents::Click),
            "change" => Ok(StimulusEvents::Change),
            "submit" => Ok(StimulusEvents::Submit),
            "input" => Ok(StimulusEvents::Input),
            "toggle" => Ok(StimulusEvents::Toggle),
            "focusout" => Ok(StimulusEvents::FocusOut),
            "focusin" => Ok(StimulusEvents::FocusIn),
            "keydown" => Ok(StimulusEvents::KeyDown),
            "keyup" => Ok(StimulusEvents::KeyUp),
            "keydown.enter" => Ok(StimulusEvents::KeyDownWithKey("enter".into())),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct StimulusAction {
    pub controller: String,
    pub method: String,
    pub action: Option<StimulusEvents>,
}

impl StimulusAction {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn controller(mut self, controller: &str) -> Self {
        self.controller = controller.to_string();
        self
    }

    pub fn method(mut self, method: &str) -> Self {
        self.method = method.to_string();
        self
    }

    pub fn action(mut self, action: StimulusEvents) -> Self {
        self.action = Some(action);
        self
    }

    pub fn new_click() -> Self {
        Self::new().action(StimulusEvents::Click)
    }

    pub fn new_change() -> Self {
        Self::new().action(StimulusEvents::Change)
    }

    pub fn new_input() -> Self {
        Self::new().action(StimulusEvents::Input)
    }

    pub fn new_focusout() -> Self {
        Self::new().action(StimulusEvents::FocusOut)
    }

    pub fn new_keydown_with_key(key: &str) -> Self {
        Self::new().action(StimulusEvents::KeyDownWithKey(key.into()))
    }
}

impl fmt::Display for StimulusAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.action {
            Some(action) => write!(f, "{}->{}#{}", action, self.controller, self.method),
            None => write!(f, "{}#{}", self.controller, self.method),
        }
    }
}

impl Render for StimulusAction {
    fn render(&self, b: &mut Buffer) -> Result<(), sailfish::RenderError> {
        if self.controller.is_empty() || self.method.is_empty() {
            return String::new().render(b);
        }
        match &self.action {
            Some(action) => format!("{}->{}#{}", action, self.controller, self.method).render(b),
            None => format!("{}#{}", self.controller, self.method).render(b),
        }
    }
}

impl FromStr for StimulusAction {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, ()> {
        let cleaned = input.replace(' ', "");
        let mut out: Vec<&str> = cleaned.split("->").collect();

        match out.len() {
            1 => {
                let mut command: Vec<&str> = out.pop().unwrap().split('#').collect();
                match command.len() {
                    2 => Ok(StimulusAction::new()
                        .method(command.pop().unwrap())
                        .controller(command.pop().unwrap())
                        .clone()),
                    _ => Err(()),
                }
            }
            2 => {
                let mut command: Vec<&str> = out.pop().unwrap().split('#').collect();
                match command.len() {
                    2 => Ok(StimulusAction::new()
                        .action(StimulusEvents::from_str(out.pop().unwrap()).unwrap())
                        .method(command.pop().unwrap())
                        .controller(command.pop().unwrap())
                        .clone()),
                    _ => Err(()),
                }
            }
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StimulusActions {
    actions: Vec<StimulusAction>,
}

impl StimulusActions {
    pub fn push(&mut self, action: StimulusAction) {
        self.actions.push(action);
    }
}

impl Render for StimulusActions {
    fn render(&self, b: &mut Buffer) -> Result<(), sailfish::RenderError> {
        let actions = self
            .actions
            .iter()
            .map(|action| action.to_string())
            .collect::<Vec<String>>();
        let actions = actions.join(" ");
        actions.render(b)
    }
}
