use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "cards/marketing/slider/template.html")]
pub struct Slider {
    title: String,
    link: String,
    image: String,
    bullets: Vec<String>,
    state: String,
    text: String,
}

impl Slider {
    pub fn new() -> Slider {
        Slider {
            title: String::new(),
            link: String::new(),
            image: String::new(),
            bullets: Vec::new(),
            state: String::new(),
            text: String::new(),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn link(mut self, link: &str) -> Self {
        self.link = link.to_string();
        self
    }

    pub fn image(mut self, image: &str) -> Self {
        self.image = image.to_string();
        self
    }

    pub fn bullets(mut self, bullets: Vec<String>) -> Self {
        self.bullets = bullets;
        self
    }

    pub fn text<T: Into<String>>(mut self, text: T) -> Self {
        self.text = text.into();
        self
    }

    pub fn active(mut self) -> Self {
        self.state = String::from("active");
        self
    }

    pub fn disabled(mut self) -> Self {
        self.state = String::from("disabled");
        self
    }
}

component!(Slider);
