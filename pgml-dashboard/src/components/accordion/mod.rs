use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "accordion/template.html")]
pub struct Accordion {
    html_contents: Vec<String>,
    html_titles: Vec<String>,
    selected: usize,
    small_titles: bool,
}

impl Accordion {
    pub fn new() -> Accordion {
        Accordion {
            html_contents: Vec::new(),
            html_titles: Vec::new(),
            selected: 0,
            small_titles: false,
        }
    }

    pub fn html_contents<S: ToString>(mut self, html_contents: Vec<S>) -> Self {
        self.html_contents = html_contents.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn html_titles<S: ToString>(mut self, html_titles: Vec<S>) -> Self {
        self.html_titles = html_titles.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn small_titles(mut self, small_titles: bool) -> Self {
        self.small_titles = small_titles;
        self
    }
}

component!(Accordion);
