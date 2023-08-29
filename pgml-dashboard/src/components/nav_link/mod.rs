use crate::components::nav::Nav;

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
