use crate::templates::models;
use crate::utils::config;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "components/box.html")]
pub struct Box<'a> {
    name: &'a str,
    value: String,
}

impl<'a> Box<'a> {
    pub fn new(name: &'a str, value: &str) -> Box<'a> {
        Box {
            name,
            value: value.to_owned(),
        }
    }
}

#[derive(Clone)]
pub struct NavLink<'a> {
    pub href: String,
    pub name: String,
    pub target_blank: bool,
    pub active: bool,
    pub nav: Option<Nav<'a>>,
    pub icon: Option<&'a str>,
}

impl<'a> NavLink<'a> {
    pub fn new(name: &str, href: &str) -> NavLink<'a> {
        NavLink {
            name: name.to_owned(),
            href: href.to_owned(),
            target_blank: false,
            active: false,
            nav: None,
            icon: None,
        }
    }

    pub fn active(mut self) -> NavLink<'a> {
        self.active = true;
        self
    }

    pub fn nav(mut self, nav: Nav<'a>) -> NavLink<'a> {
        self.nav = Some(nav);
        self
    }

    pub fn icon(mut self, icon: &'a str) -> NavLink<'a> {
        self.icon = Some(icon);
        self
    }
}

#[derive(TemplateOnce, Clone)]
#[template(path = "components/nav.html")]
pub struct Nav<'a> {
    pub links: Vec<NavLink<'a>>,
}

impl<'a> Nav<'a> {
    pub fn render(links: Vec<NavLink<'a>>) -> String {
        Nav { links }.render_once().unwrap()
    }
}

#[derive(TemplateOnce)]
#[template(path = "components/breadcrumbs.html")]
pub struct Breadcrumbs<'a> {
    pub links: Vec<NavLink<'a>>,
}

impl<'a> Breadcrumbs<'a> {
    pub fn render(links: Vec<NavLink<'a>>) -> String {
        Breadcrumbs { links }.render_once().unwrap()
    }
}

#[derive(TemplateOnce)]
#[template(path = "components/boxes.html")]
pub struct Boxes<'a> {
    pub boxes: Vec<Box<'a>>,
}

#[derive(TemplateOnce)]
#[template(path = "layout/nav/top.html")]
pub struct Navbar {
    pub current_user: Option<models::User>,
    pub standalone_dashboard: bool,
}

impl Navbar {
    pub fn render(user: Option<models::User>) -> String {
        Navbar {
            current_user: user,
            standalone_dashboard: config::standalone_dashboard(),
        }
        .render_once()
        .unwrap()
    }
}
