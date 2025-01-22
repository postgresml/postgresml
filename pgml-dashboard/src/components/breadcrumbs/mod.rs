use crate::components::dropdown::{Dropdown, DropdownItems};
use crate::components::NavLink;
use crate::components::StaticNavLink;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Clone, Default)]
#[template(path = "breadcrumbs/template.html")]
pub struct Breadcrumbs<'a> {
    pub organizations: Vec<StaticNavLink>,
    pub databases: Vec<StaticNavLink>,
    pub path: Vec<NavLink<'a>>,
}

impl<'a> Breadcrumbs<'a> {
    pub fn new(path: Vec<NavLink<'a>>, organizations: Vec<StaticNavLink>, databases: Vec<StaticNavLink>) -> Self {
        Breadcrumbs {
            path,
            databases,
            organizations,
        }
    }
}

component!(Breadcrumbs, 'a);
