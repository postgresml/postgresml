use crate::components::cms::IndexLink;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/left_nav/docs/template.html")]
pub struct Docs {
    index: Vec<IndexLink>,
    mobile: bool,
}

impl Docs {
    pub fn new(index: &Vec<IndexLink>) -> Docs {
        Docs {
            index: index.clone(),
            mobile: false,
        }
    }

    pub fn for_mobile(mut self) -> Docs {
        self.mobile = true;
        self
    }
}

component!(Docs);
