use crate::templates::docs::TocLink;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/toc/template.html")]
pub struct Toc {
    toc_links: Vec<TocLink>,
}

impl Toc {
    pub fn new(links: &Vec<TocLink>) -> Toc {
        Toc {
            toc_links: links.clone(),
        }
    }
}

component!(Toc);
