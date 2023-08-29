use crate::components::component;
use crate::components::{StaticNav, StaticNavLink};
use crate::utils::config;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "navbar_web_app/template.html")]
pub struct NavbarWebApp {
    pub standalone_dashboard: bool,
    pub links: Vec<StaticNavLink>,
    pub account_management_nav: StaticNav,
}

impl NavbarWebApp {
    pub fn render(links: Vec<StaticNavLink>, account_management_nav: StaticNav) -> String {
        NavbarWebApp {
            standalone_dashboard: config::standalone_dashboard(),
            links,
            account_management_nav,
        }
        .render_once()
        .unwrap()
    }
}

component!(NavbarWebApp);
