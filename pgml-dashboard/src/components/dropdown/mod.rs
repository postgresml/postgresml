use crate::components::component;
use sailfish::TemplateOnce;

use crate::components::StaticNavLink;

#[derive(TemplateOnce, Default)]
#[template(path = "dropdown/template.html")]
pub struct Dropdown {
    /// The currently selected value.
    value: String,

    /// The list of dropdown links to render.
    links: Vec<StaticNavLink>,

    /// Whether or not the dropdown is collapsble.
    collapsable: bool,
}

impl Dropdown {
    pub fn new(links: Vec<StaticNavLink>) -> Self {
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

    pub fn collapsable(mut self) -> Self {
        self.collapsable = true;
        self
    }
}

component!(Dropdown);
