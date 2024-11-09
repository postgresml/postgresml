use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/left_nav/menu/index/template.html")]
pub struct Index {
    pub nav: Vec<Component>,
}

impl Index {
    pub fn new(nav: Vec<Component>) -> Index {
        Index { nav }
    }
}

component!(Index);
