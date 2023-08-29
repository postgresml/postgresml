use crate::components::component;
use sailfish::TemplateOnce;

use crate::components::StaticNav;

#[derive(TemplateOnce)]
#[template(path = "left_nav_web_app/template.html")]
pub struct LeftNavWebApp {
    pub upper_nav: StaticNav,
    pub lower_nav: StaticNav,
    pub dropdown_nav: StaticNav,
}

impl LeftNavWebApp {
    pub fn render(upper_nav: StaticNav, lower_nav: StaticNav, dropdown_nav: StaticNav) -> String {
        LeftNavWebApp {
            upper_nav,
            lower_nav,
            dropdown_nav,
        }
        .render_once()
        .unwrap()
    }
}

component!(LeftNavWebApp);
