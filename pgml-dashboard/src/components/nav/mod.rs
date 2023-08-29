use crate::components::component;
use crate::components::nav_link::NavLink;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Clone, Default, Debug)]
#[template(path = "nav/template.html")]
pub struct Nav<'a> {
    pub links: Vec<NavLink<'a>>,
}

impl<'a> Nav<'a> {
    pub fn render(links: Vec<NavLink<'a>>) -> String {
        Nav { links }.render_once().unwrap()
    }

    pub fn add_link(&mut self, link: NavLink<'a>) -> &mut Self {
        self.links.push(link);
        self
    }
}

component!(Nav, 'a);
