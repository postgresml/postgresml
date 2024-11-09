use crate::components::NavLink;
use crate::components::StaticNav;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "breadcrumbs/template.html")]
pub struct Breadcrumbs<'a> {
    pub dropdown_1: StaticNav,
    pub dropdown_2: StaticNav,
    pub links: Vec<NavLink<'a>>,
}

impl<'a> Breadcrumbs<'a> {
    pub fn render(dropdown_1: StaticNav, dropdown_2: StaticNav, links: Vec<NavLink<'a>>) -> String {
        Breadcrumbs {
            dropdown_1,
            dropdown_2,
            links,
        }
        .render_once()
        .unwrap()
    }
}

component!(Breadcrumbs, 'a);
