use crate::components::NavLink;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "breadcrumbs/template.html")]
pub struct Breadcrumbs<'a> {
    pub links: Vec<NavLink<'a>>,
}

impl<'a> Breadcrumbs<'a> {
    pub fn render(links: Vec<NavLink<'a>>) -> String {
        Breadcrumbs { links }.render_once().unwrap()
    }
}

component!(Breadcrumbs, 'a);
