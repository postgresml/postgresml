use pgml_components::component;
use pgml_components::Component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "sections/split/template.html")]
pub struct Split {
    eyebrow: Component,
    title: Component,
    display_area: Component,
    with_navbar: bool,
}

impl Split {
    pub fn new() -> Split {
        Split {
            eyebrow: Component::from(String::from("")),
            title: Component::from(String::from("")),
            display_area: Component::from(String::from("")),
            with_navbar: false,
        }
    }

    pub fn eyebrow(mut self, eyebrow: Component) -> Split {
        self.eyebrow = eyebrow;
        self
    }

    pub fn title(mut self, title: Component) -> Split {
        self.title = title;
        self
    }

    pub fn display_area(mut self, display_area: Component) -> Split {
        self.display_area = display_area;
        self
    }

    pub fn with_navbar(mut self) -> Split {
        self.with_navbar = true;
        self
    }
}

component!(Split);
