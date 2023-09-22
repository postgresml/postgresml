use crate::components::navigation::dropdown_link::DropdownLink;
use crate::components::stimulus::stimulus_target::StimulusTarget;
use pgml_components::component;
use pgml_components::Component;
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

    /// The list of dropdown items to render.
    items: Vec<Component>,

    /// Position of the dropdown menu.
    offset: String,

    /// Whether or not the dropdown is collapsable.
    collapsable: bool,
    offset_collapsed: String,

    /// Where the dropdown menu should appear
    menu_position: String,
    expandable: bool,

    /// target to control value
    value_target: StimulusTarget,
}

impl Dropdown {
    pub fn new() -> Self {
        Dropdown {
            items: Vec::new(),
            value: DropdownValue::Text("Dropdown".to_owned().into()),
            offset: "0, 10".to_owned(),
            offset_collapsed: "68, -44".to_owned(),
            menu_position: "".to_owned(),
            ..Default::default()
        }
    }

    pub fn nav(links: Vec<StaticNavLink>) -> Self {
        let binding = links
            .iter()
            .filter(|link| link.active)
            .collect::<Vec<&StaticNavLink>>();

        let active = binding.first();
        let value = if let Some(active) = active {
            active.name.to_owned()
        } else {
            "Dropdown Nav".to_owned()
        };

        let mut items = Vec::new();
        for link in links {
            let item = DropdownLink::new(link);
            items.push(item.into());
        }

        Dropdown {
            items,
            value: DropdownValue::Text(value.into()),
            offset: "0, 10".to_owned(),
            offset_collapsed: "68, -44".to_owned(),
            menu_position: "".to_owned(),
            ..Default::default()
        }
    }

    pub fn items(mut self, items: Vec<Component>) -> Self {
        self.items = items;
        self
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

    pub fn value_target(mut self, value_target: StimulusTarget) -> Self {
        self.value_target = value_target;
        self
    }
}

component!(Dropdown);
