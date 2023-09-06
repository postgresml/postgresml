use crate::components::StaticNav;
use pgml_components::component;
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
