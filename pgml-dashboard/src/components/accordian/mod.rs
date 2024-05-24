use pgml_components::component;
use sailfish::TemplateOnce;

// This component will probably not work very well if two are on the same page at once. We can get
// around it if we include some randomness with the data values in the template.html but that
// doesn't feel very clean so I will leave this problem until we have need to fix it or a better
// idea of how to
#[derive(TemplateOnce, Default)]
#[template(path = "accordian/template.html")]
pub struct Accordian {
    html_contents: Vec<String>,
    html_titles: Vec<String>,
    selected: usize,
    small_titles: bool,
}

impl Accordian {
    pub fn new() -> Accordian {
        Accordian {
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

component!(Accordian);
