use crate::components::navigation::dropdown_link::DropdownLink;
use crate::components::stimulus::stimulus_target::StimulusTarget;
use pgml_components::component;
use pgml_components::Component;
use sailfish::TemplateOnce;

use crate::components::StaticNavLink;

pub enum DropdownValue {
    Icon(Component),
    Text(Component),
    None,
}

impl Default for DropdownValue {
    fn default() -> Self {
        DropdownValue::Text("Menu".into())
    }
}

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/dropdown/block/index/dropdown_items.html")]
pub struct DropdownItems {
    items: Vec<Component>,
}

impl DropdownItems {
    pub fn new(items: Vec<Component>) -> Self {
        DropdownItems { items }
    }
}

component!(DropdownItems);

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/dropdown/block/index/dropdown_frame.html")]
pub struct DropdownFrame {
    src: Option<String>,
    id: String,
    content: Component,
}

impl DropdownFrame {
    pub fn rendered(id: impl ToString, content: Component) -> Self {
        DropdownFrame {
            src: None,
            id: id.to_string(),
            content,
        }
    }

    pub fn new(id: impl ToString, src: impl ToString) -> Self {
        DropdownFrame {
            src: Some(src.to_string()),
            id: id.to_string(),
            content: "".into(),
        }
    }
}

component!(DropdownFrame);

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/dropdown/block/index/template.html")]
pub struct Index {
    /// The currently selected value.
    value: DropdownValue,

    /// The list of dropdown items to render.
    items: Component,

    /// Position of the dropdown menu.
    offset: String,

    /// Whether or not the dropdown responds to horizontal collapse, i.e. in product left nav.
    collapsable: bool,
    offset_collapsed: String,

    /// Where the dropdown menu should appear
    menu_position: String,
    expandable: bool,

    /// target to control value
    value_target: StimulusTarget,

    /// If the dropdown should be shown
    show: String,
}

impl Index {
    pub fn new() -> Index {
        Index {
            items: DropdownItems::default().into(),
            value: DropdownValue::Text("Dropdown".to_owned().into()),
            offset: "0, 10".to_owned(),
            offset_collapsed: "68, -44".to_owned(),
            menu_position: "".to_owned(),
            ..Default::default()
        }
    }

    pub fn new_no_button() -> Self {
        Index {
            value: DropdownValue::None,
            ..Self::new()
        }
    }

    pub fn nav(links: Vec<StaticNavLink>) -> Self {
        let binding = links.iter().filter(|link| link.active).collect::<Vec<&StaticNavLink>>();

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

        Index {
            items: DropdownItems::new(items).into(),
            value: DropdownValue::Text(value.into()),
            offset: "0, 10".to_owned(),
            offset_collapsed: "68, -44".to_owned(),
            menu_position: "".to_owned(),
            ..Default::default()
        }
    }

    pub fn items(mut self, items: Vec<Component>) -> Self {
        self.items = DropdownItems::new(items).into();
        self
    }

    pub fn frame(mut self, id: impl ToString, src: impl ToString) -> Self {
        self.items = DropdownFrame::new(id, src).into();

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

    pub fn show(mut self) -> Self {
        self.show = "show".into();
        self
    }
}

component!(Index);
