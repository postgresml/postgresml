use crate::components::navigation::Anchor;
use crate::components::component::Component;
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

impl From<StaticNavLink> for Anchor {
    fn from(link: StaticNavLink) -> Anchor {
        let mut anchor = Anchor::new(Component::from(link.name), link.href);
        if link.active {
            anchor = anchor.active();
        }
        if link.disabled {
            anchor = anchor.disabled();
        }
        anchor
    }
}
