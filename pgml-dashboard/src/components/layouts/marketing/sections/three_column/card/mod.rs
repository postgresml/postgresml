use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "layouts/marketing/sections/three_column/card/template.html")]
pub struct Card {
    pub title: Component,
    pub icon: String,
    pub color: String,
    pub paragraph: Component,
}

impl Card {
    pub fn new() -> Card {
        Card {
            title: "title".into(),
            icon: "home".into(),
            color: "red".into(),
            paragraph: "paragraph".into(),
        }
    }

    pub fn set_title(mut self, title: Component) -> Self {
        self.title = title;
        self
    }

    pub fn set_icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_string();
        self
    }

    pub fn set_color_red(mut self) -> Self {
        self.color = "red".into();
        self
    }

    pub fn set_color_orange(mut self) -> Self {
        self.color = "orange".into();
        self
    }

    pub fn set_color_purple(mut self) -> Self {
        self.color = "purple".into();
        self
    }

    pub fn set_paragraph(mut self, paragraph: Component) -> Self {
        self.paragraph = paragraph;
        self
    }
}

component!(Card);
