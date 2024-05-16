use pgml_components::{component, Component};
use sailfish::TemplateOnce;
use std::fmt;

#[derive(PartialEq, Eq, Default, Clone)]
pub enum Color {
    #[default]
    Green,
    Blue,
    Orange,
    Pink,
    Purple,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Color::Green => write!(f, "green"),
            Color::Blue => write!(f, "blue"),
            Color::Orange => write!(f, "orange"),
            Color::Pink => write!(f, "pink"),
            Color::Purple => write!(f, "purple"),
        }
    }
}

#[derive(TemplateOnce, Default)]
#[template(path = "lists/item/template.html")]
pub struct Item {
    value: String,
    color: Color,
    alt_item_indicator: Option<Component>,
}

impl Item {
    pub fn new() -> Item {
        Item {
            value: String::from("Your list item"),
            color: Color::Green,
            alt_item_indicator: None,
        }
    }

    pub fn value(mut self, value: &str) -> Item {
        self.value = value.into();
        self
    }

    pub fn color(mut self, color: Color) -> Item {
        self.color = color;
        self
    }

    pub fn alt_item_indicator(mut self, indicator: Component) -> Item {
        self.alt_item_indicator = Some(indicator);
        self
    }
}

component!(Item);
