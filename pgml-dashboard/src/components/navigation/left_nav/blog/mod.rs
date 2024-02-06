use crate::components::cms::IndexLink;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/left_nav/blog/template.html")]
pub struct Blog {
    nav_title: String,
    nav_links: Vec<IndexLink>,
}

impl Blog {
    pub fn new(links: &Vec<IndexLink>) -> Blog {
        Blog {
            nav_links: links.clone(),
            ..Default::default()
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.nav_title = title.to_owned();
        self
    }
}

component!(Blog);
