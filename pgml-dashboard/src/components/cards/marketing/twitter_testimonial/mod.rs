use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "cards/marketing/twitter_testimonial/template.html")]
pub struct TwitterTestimonial {
    statement: String,
    image: String,
    name: String,
    handle: String,
    verified: bool,
}

impl TwitterTestimonial {
    pub fn new() -> TwitterTestimonial {
        TwitterTestimonial {
            statement: String::from("src/components/cards/marketing/twitter_testimonial"),
            image: String::new(),
            name: String::new(),
            handle: String::new(),
            verified: false,
        }
    }

    pub fn statement(mut self, statement: &str) -> Self {
        self.statement = statement.to_owned();
        self
    }

    pub fn image(mut self, image: &str) -> Self {
        self.image = image.to_owned();
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    pub fn handle(mut self, handle: &str) -> Self {
        self.handle = handle.to_owned();
        self
    }

    pub fn verified(mut self) -> Self {
        self.verified = true;
        self
    }
}

component!(TwitterTestimonial);
