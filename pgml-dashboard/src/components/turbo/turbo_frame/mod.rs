use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "turbo/turbo_frame/template.html")]
pub struct TurboFrame {
    src: Component,
    target_id: String,
    content: Option<Component>,
    attributes: Vec<String>,
}

impl TurboFrame {
    pub fn new() -> TurboFrame {
        TurboFrame {
            src: Component::from(""),
            target_id: "".to_string(),
            content: None,
            attributes: vec![],
        }
    }

    pub fn set_src(mut self, src: Component) -> Self {
        self.src = src;
        self
    }

    pub fn set_target_id(mut self, target_id: &str) -> Self {
        self.target_id = target_id.to_string();
        self
    }

    pub fn set_content(mut self, content: Component) -> Self {
        self.content = Some(content);
        self
    }

    pub fn add_attribute(mut self, attribute: &str) -> Self {
        self.attributes.push(attribute.to_string());
        self
    }
}

component!(TurboFrame);
