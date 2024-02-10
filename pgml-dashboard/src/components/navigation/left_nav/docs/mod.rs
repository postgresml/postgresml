use crate::components::cms::IndexLink;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/left_nav/docs/template.html")]
pub struct Docs {
    index: Vec<IndexLink>,
}

impl Docs {
    pub fn new(index: &Vec<IndexLink>) -> Docs {
        Docs { index: index.clone() }
    }
}

component!(Docs);
