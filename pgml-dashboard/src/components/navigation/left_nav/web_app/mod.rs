use crate::components::{StaticNav, StaticNavLink};
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(Default, Debug, Clone)]
pub struct Menu {
    pub back: Option<StaticNavLink>,
    pub items: StaticNav,
}

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/left_nav/web_app/template.html")]
pub struct WebApp {
    pub id: Option<String>,
    pub menu: Menu,
}

impl WebApp {
    pub fn new(menu: Menu) -> WebApp {
        WebApp { id: None, menu }
    }

    pub fn id(mut self, id: &str) -> WebApp {
        self.id = Some(id.to_string());
        self
    }
}

component!(WebApp);
