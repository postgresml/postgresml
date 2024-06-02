use crate::components::static_nav_link::StaticNavLink as NavLink;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/navbar/marketing_link/template.html")]
pub struct MarketingLink {
    name: String,
    link: Option<NavLink>,
    links: Vec<NavLink>,
    links_col2: Vec<NavLink>,
    title_col1: Option<String>,
    title_col2: Option<String>,
}

impl MarketingLink {
    pub fn new() -> MarketingLink {
        MarketingLink {
            name: String::from("Link Name"),
            links: Vec::new(),
            links_col2: Vec::new(),
            link: None,
            title_col1: None,
            title_col2: None,
        }
    }

    pub fn links(mut self, links: Vec<NavLink>) -> MarketingLink {
        self.links = links;
        self.link = None;
        self
    }

    pub fn name(mut self, name: &str) -> MarketingLink {
        self.name = name.to_owned();
        self
    }

    pub fn link(mut self, link: NavLink) -> MarketingLink {
        self.link = Some(link);
        self
    }

    pub fn links_col2(mut self, links: Vec<NavLink>) -> MarketingLink {
        self.links_col2 = links;
        self
    }

    pub fn title_col1(mut self, title: &str) -> MarketingLink {
        self.title_col1 = Some(title.to_owned());
        self
    }

    pub fn title_col2(mut self, title: &str) -> MarketingLink {
        self.title_col2 = Some(title.to_owned());
        self
    }
}

component!(MarketingLink);
