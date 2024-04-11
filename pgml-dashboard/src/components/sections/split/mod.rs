//! Left/right split used in onboarding, signup, careers, etc.

use pgml_components::component;
use pgml_components::Component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "sections/split/template.html")]
pub struct Split {
    greeting_area: Component,
    display_area: Component,
    with_navbar: bool,
}

// Greeting with its own styling.
#[derive(TemplateOnce, Default, Clone)]
#[template(path = "sections/split/greeting.html")]
pub struct Greeting {
    eyebrow: Component,
    title: Component,
}

component!(Greeting);

impl Greeting {
    pub fn new(eyebrow: Component, title: Component) -> Greeting {
        Greeting { eyebrow, title }
    }
}

impl Split {
    pub fn new() -> Split {
        Split {
            greeting_area: Component::default(),
            display_area: Component::default(),
            with_navbar: false,
        }
    }

    // Set the greeting.
    pub fn greeting(mut self, eyebrow: Component, title: Component) -> Split {
        self.greeting_area = Greeting::new(eyebrow, title).into();
        self
    }

    // Set whatever you want on the left side of the display.
    pub fn greeting_area(mut self, greeting_area: Component) -> Split {
        self.greeting_area = greeting_area;
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
