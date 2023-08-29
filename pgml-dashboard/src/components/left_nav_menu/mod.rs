use crate::components::component;
use crate::components::StaticNav;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "left_nav_menu/template.html")]
pub struct LeftNavMenu {
    pub nav: StaticNav,
}

impl LeftNavMenu {
    pub fn new(nav: StaticNav) -> LeftNavMenu {
        LeftNavMenu { nav }
    }
}

component!(LeftNavMenu);
