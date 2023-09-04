use crate::components::component;
use crate::components::component::Component;
use sailfish::TemplateOnce;

use crate::components::StaticNavLink;

pub enum DropdownValue {
    Icon(Component),
    Text(Component),
}

impl Default for DropdownValue {
    fn default() -> Self {
        DropdownValue::Text("Menu".into())
    }
}

#[derive(TemplateOnce, Default)]
#[template(path = "dropdown/template.html")]
pub struct Dropdown {
    /// The currently selected value.
    value: DropdownValue,

    /// The list of dropdown links to render.
    links: Vec<StaticNavLink>,

    /// Position of the dropdown menu.
    offset: String,

    /// Whether or not the dropdown is collapsble.
    collapsable: bool,
    offset_collapsed: String,

    /// Where the dropdown menu should appear
    menu_position: String,
    expandable: bool,
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
            value: DropdownValue::Text(value.into()),
            offset: "0, 10".to_owned(),
            offset_collapsed: "68, -44".to_owned(),
            menu_position: "".to_owned(),
            ..Default::default()
        }
    }

    pub fn text(mut self, value: Component) -> Self {
        self.value = DropdownValue::Text(value);
        self
    }

    pub fn icon(mut self, icon: Component) -> Self {
        self.value = DropdownValue::Icon(icon);
        self
    }

    pub fn collapsable(mut self) -> Self {
        self.collapsable = true;
        self
    }

    pub fn menu_end(mut self) -> Self {
        self.menu_position = "dropdown-menu-end".to_owned();
        self
    }

    pub fn menu_start(mut self) -> Self {
        self.menu_position = "dropdown-menu-start".to_owned();
        self
    }

    pub fn offset(mut self, offset: &str) -> Self {
        self.offset = offset.to_owned();
        self
    }

    pub fn offset_collapsed(mut self, offset: &str) -> Self {
        self.offset_collapsed = offset.to_owned();
        self
    }

    pub fn expandable(mut self) -> Self {
        self.expandable = true;
        self
    }
}

component!(Dropdown);
