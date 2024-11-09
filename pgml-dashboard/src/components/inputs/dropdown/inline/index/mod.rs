use crate::components::navigation::dropdown_link::DropdownLink;
use crate::components::{StaticNav, StaticNavLink};
use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/dropdown/inline/index/template.html")]
pub struct Index {
    title: String,
    items: Vec<Component>,
}

impl Index {
    pub fn new() -> Index {
        Index {
            title: String::new(),
            items: vec![],
        }
    }

    pub fn set_nav_items(mut self, items: StaticNav) -> Self {
        let active = items
            .links
            .clone()
            .iter()
            .filter(|item| item.active)
            .collect::<Vec<&StaticNavLink>>()
            .first()
            .map(|x| x.name.to_string())
            .unwrap_or("Dropdown Nav".to_string());

        self.title = active;
        self.items = items
            .links
            .iter()
            .map(|item| DropdownLink::new(item.clone()).into())
            .collect::<Vec<Component>>();
        self
    }
}

component!(Index);
