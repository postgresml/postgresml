use crate::frontend::components::Component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "backend/controllers/templates/frame.rs.tpl")]
pub struct Frame {
    component: Component,
}

impl Frame {
    pub fn new(component: &Component) -> Self {
        Frame { component: component.clone() }
    }
}
