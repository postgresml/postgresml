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
        for item in index.clone() {
            if item.children.is_empty() {
                println!("no children");
            }
            println!("index header: {:?}\n", item);
        }
        Docs { index: index.clone() }
    }
}

component!(Docs);
