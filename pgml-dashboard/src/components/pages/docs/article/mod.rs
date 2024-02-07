use crate::docs::TocLink;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "pages/docs/article/template.html")]
pub struct Article {
    toc_links: Vec<TocLink>,
}

impl Article {
    pub fn new() -> Article {
        Article { ..Default::default() }
    }

    pub fn toc_links(mut self, toc_links: &Vec<TocLink>) -> Self {
        self.toc_links = toc_links.clone();
        self
    }
}

component!(Article);
