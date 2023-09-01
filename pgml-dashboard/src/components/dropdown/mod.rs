use crate::components::component;
use sailfish::TemplateOnce;

use crate::components::StaticNavLink;

#[derive(TemplateOnce, Default)]
#[template(path = "dropdown/template.html")]
pub struct Dropdown {
    value: String,
    links: Vec<StaticNavLink>,
}

impl Dropdown {
    pub fn new(links: Vec<StaticNavLink>) -> Dropdown {
        let binding = links
            .iter()
            .filter(|link| link.active)
            .collect::<Vec<&StaticNavLink>>();
        let active = binding.first();
        let value = if let Some(active) = active {
            active.name.to_owned()
        } else {
            "Menu".to_owned()
        };
        Dropdown {
            links,
            value,
            ..Default::default()
        }
    }
}

component!(Dropdown);
