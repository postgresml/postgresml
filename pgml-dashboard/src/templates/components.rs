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

#[derive(Debug, Clone, Default)]
pub struct StaticNav {
    pub links: Vec<StaticNavLink>,
}

impl StaticNav {
    pub fn add_link(&mut self, link: StaticNavLink) {
        self.links.push(link);
    }

    pub fn get_active(self) -> StaticNavLink {
        match self.links.iter().find(|item| item.active) {
            Some(item) => item.clone(),
            None => StaticNavLink {
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StaticNavLink {
    pub name: String,
    pub href: String,
    pub active: bool,
    pub disabled: bool,
    pub icon: Option<String>,
    pub hide_for_lg_screens: bool,
}

impl StaticNavLink {
    pub fn new(name: String, href: String) -> StaticNavLink {
        StaticNavLink {
            name,
            href,
            active: false,
            disabled: false,
            icon: None,
            hide_for_lg_screens: false,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn hide_for_lg_screens(mut self, hide: bool) -> Self {
        self.hide_for_lg_screens = hide;
        self
    }
}

#[derive(TemplateOnce)]
#[template(path = "components/left_nav_menu.html")]
pub struct LeftNavMenu {
    pub nav: StaticNav,
}
