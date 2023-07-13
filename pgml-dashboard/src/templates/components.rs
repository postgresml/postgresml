use std::collections::HashMap;

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

#[derive(Clone, Debug)]
pub struct NavLink<'a> {
    pub href: String,
    pub name: String,
    pub target_blank: bool,
    pub active: bool,
    pub nav: Option<Nav<'a>>,
    pub icon: Option<&'a str>,
    pub disabled: bool,
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
            disabled: false,
        }
    }

    pub fn active(mut self) -> NavLink<'a> {
        self.active = true;
        self
    }

    pub fn disable(mut self, disabled: bool) -> NavLink<'a> {
        self.disabled = disabled;
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

#[derive(TemplateOnce, Clone, Default, Debug)]
#[template(path = "components/nav.html")]
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

#[derive(TemplateOnce)]
#[template(path = "layout/nav/left_web_app.html")]
pub struct LeftNavWebApp<'a> {
    pub upper_nav: Nav<'a>,
    pub lower_nav: Nav<'a>,
    pub dropdown_nav: Nav<'a>,
}

impl<'a> LeftNavWebApp<'a> {
    pub fn render(upper_nav: Nav, lower_nav: Nav, dropdown_nav: Nav) -> String {
        LeftNavWebApp {
            upper_nav,
            lower_nav,
            dropdown_nav,
        }
        .render_once()
        .unwrap()
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

#[derive(TemplateOnce)]
#[template(path = "layout/nav/top_web_app.html")]
pub struct NavbarWebApp<'a> {
    pub current_user: Option<models::User>,
    pub standalone_dashboard: bool,
    pub links: Vec<NavLink<'a>>,
}

impl<'a> NavbarWebApp<'a> {
    pub fn render(user: Option<models::User>, links: Vec<NavLink<'a>>) -> String {
        NavbarWebApp {
            current_user: user,
            standalone_dashboard: config::standalone_dashboard(),
            links,
        }
        .render_once()
        .unwrap()
    }
}

#[derive(TemplateOnce)]
#[template(path = "components/github_icon.html")]
pub struct GithubIcon {
    pub show_stars: bool,
}

#[derive(TemplateOnce)]
#[template(path = "components/postgres_logo.html")]
pub struct PostgresLogo {
    link: String,
}
