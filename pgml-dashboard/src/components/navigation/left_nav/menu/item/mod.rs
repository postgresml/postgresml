use crate::components::StaticNavLink;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/left_nav/menu/item/template.html")]
pub struct Item {
    name: String,
    link: Option<String>,
    icon: Option<String>,
    active: bool,
    disabled: bool,
    actions: Option<String>,
    hide_for_lg_screens: bool,
}

impl Item {
    pub fn new() -> Item {
        Item {
            name: String::new(),
            link: None,
            icon: None,
            active: false,
            disabled: false,
            actions: None,
            hide_for_lg_screens: false,
        }
    }

    pub fn from_static_nav(nav: &StaticNavLink) -> Item {
        Item {
            name: nav.name.clone(),
            link: Some(nav.href.clone()),
            icon: nav.icon.clone(),
            active: nav.active,
            disabled: nav.disabled,
            actions: None,
            hide_for_lg_screens: nav.hide_for_lg_screens,
        }
    }

    pub fn name(mut self, name: &str) -> Item {
        self.name = name.to_string();
        self
    }

    pub fn link(mut self, link: &str) -> Item {
        self.link = Some(link.to_string());
        self
    }

    pub fn icon(mut self, icon: &str) -> Item {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn active(mut self, active: bool) -> Item {
        self.active = active;
        self
    }

    pub fn actions(mut self, actions: &str) -> Item {
        self.actions = Some(actions.to_string());
        self
    }

    pub fn hide_for_lg_screens(mut self, hide: bool) -> Item {
        self.hide_for_lg_screens = hide;
        self
    }
}

component!(Item);
