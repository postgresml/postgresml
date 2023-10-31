use crate::components::static_nav_link::StaticNavLink as NavLink;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/navbar/marketing_link/template.html")]
pub struct MarketingLink {
    name: String,
    link: Option<NavLink>,
    links: Vec<NavLink>,
}

impl MarketingLink {
    pub fn new() -> MarketingLink {
        MarketingLink {
            name: String::from("Link Name"),
            links: Vec::new(),
            link: None,
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
}

component!(MarketingLink);
