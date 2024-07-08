use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "accordion/template.html")]
pub struct Accordion {
    html_contents: Vec<Component>,
    html_titles: Vec<Component>,
    selected: usize,
    title_size: String,
}

impl Accordion {
    pub fn new() -> Accordion {
        Accordion {
            html_contents: Vec::new(),
            html_titles: Vec::new(),
            selected: 0,
            title_size: "h5".to_string(),
        }
    }

    pub fn html_contents(mut self, html_contents: Vec<Component>) -> Self {
        self.html_contents = html_contents;
        self
    }

    pub fn html_titles(mut self, html_titles: Vec<Component>) -> Self {
        self.html_titles = html_titles;
        self
    }

    pub fn set_title_size_body(mut self) -> Self {
        self.title_size = "body-regular-text".to_string();
        self
    }

    pub fn set_title_size_header(mut self, title_size: i32) -> Self {
        match title_size {
            1 => self.title_size = "h1".to_string(),
            2 => self.title_size = "h2".to_string(),
            3 => self.title_size = "h3".to_string(),
            4 => self.title_size = "h4".to_string(),
            5 => self.title_size = "h5".to_string(),
            6 => self.title_size = "h6".to_string(),
            _ => self.title_size = "h5".to_string(),
        }
        self
    }
}

component!(Accordion);
