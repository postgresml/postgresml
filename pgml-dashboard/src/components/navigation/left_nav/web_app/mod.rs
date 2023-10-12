use crate::components::StaticNav;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/left_nav/web_app/template.html")]
pub struct WebApp {
    pub upper_nav: StaticNav,
    pub lower_nav: StaticNav,
    pub dropdown_nav: StaticNav,
}

impl WebApp {
    pub fn new(upper_nav: StaticNav, lower_nav: StaticNav, dropdown_nav: StaticNav) -> WebApp {
        WebApp {
            upper_nav,
            lower_nav,
            dropdown_nav,
        }
    }
}

component!(WebApp);
