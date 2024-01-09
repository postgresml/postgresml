use sailfish::runtime::{Buffer, Render};

#[derive(Debug, Clone, Default)]
pub struct StimulusTarget {
    pub controller: Option<String>,
    pub name: Option<String>,
}

impl StimulusTarget {
    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn controller(mut self, controller: &str) -> Self {
        self.controller = Some(controller.to_string());
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
}

impl Render for StimulusTarget {
    fn render(&self, b: &mut Buffer) -> Result<(), sailfish::RenderError> {
        match (self.controller.as_ref(), self.name.as_ref()) {
            (Some(controller), Some(name)) => format!("data-{}-target=\"{}\"", controller, name).render(b),
            _ => String::new().render(b),
        }
    }
}
