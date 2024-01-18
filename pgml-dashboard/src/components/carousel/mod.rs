use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "carousel/template.html")]
pub struct Carousel {
    items: Vec<String>,
}

impl Carousel {
    pub fn new(items: Vec<String>) -> Carousel {
        Carousel { items }
    }
}

component!(Carousel);
