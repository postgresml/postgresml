use crate::components::StaticNav;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/left_nav/web_app/template.html")]
pub struct WebApp {
    pub upper_nav: StaticNav,
    pub id: Option<String>,
}

impl WebApp {
    pub fn new(upper_nav: StaticNav) -> WebApp {
        WebApp { upper_nav, id: None }
    }

    pub fn id(mut self, id: &str) -> WebApp {
        self.id = Some(id.to_string());
        self
    }
}

component!(WebApp);
