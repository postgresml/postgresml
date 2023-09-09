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


#[derive(TemplateOnce)]
#[template(path = "backend/controllers/templates/routes.rs.tpl")]
pub struct Routes {
    components: Vec<Component>,
}

impl Routes {
    pub fn new(components: &[Component]) -> Self {
        Routes { components: components.to_vec() }
    }
}
