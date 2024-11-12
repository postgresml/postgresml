use crate::components::{StaticNav, StaticNavLink};
use pgml_components::component;
use sailfish::TemplateOnce;

use crate::components::navigation::left_nav::menu::Index as LeftNavMenu;
use crate::components::navigation::left_nav::menu::Item as LeftNavItem;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/left_nav/web_app/template.html")]
pub struct WebApp {
    pub upper_nav: LeftNavMenu,
    pub lower_nav: LeftNavMenu,
    pub id: Option<String>,
    pub home_link: String,
}

impl WebApp {
    pub fn new(mut upper_nav: StaticNav) -> WebApp {
        let home_link = upper_nav.links.remove(0).href;

        let mut upper_nav_items = vec![];
        for item in upper_nav.links {
            upper_nav_items.push(LeftNavItem::from_static_nav(&item).into());
        }

        // Lower nave does not change from page to page
        let lower_nav_items = vec![
                LeftNavItem::from_static_nav(&StaticNavLink::new("Docs".into(), "/Docs".into()).icon("description")).into(),
                LeftNavItem::from_static_nav(&StaticNavLink::new("Support".into(), "/support".into()).icon("build_circle")).into(),
                LeftNavItem::from_static_nav(&StaticNavLink::new("Sign Out".into(), "/logout".into()).icon("arrow_outward")).into(),
                LeftNavItem::new().name("Search").icon("search").actions(
                    "type=\"text\" name=\"search\" data-bs-toggle=\"modal\" data-bs-target=\"#search\" autocomplete=\"off\" data-search-target=\"searchTrigger\" data-action=\"search#openSearch\""
                ).into(),
            ];

        let upper_nav = LeftNavMenu::new(upper_nav_items);
        let lower_nav = LeftNavMenu::new(lower_nav_items);

        WebApp {
            upper_nav,
            lower_nav,
            id: None,
            home_link,
        }
    }

    pub fn id(mut self, id: &str) -> WebApp {
        self.id = Some(id.to_string());
        self
    }
}

component!(WebApp);
