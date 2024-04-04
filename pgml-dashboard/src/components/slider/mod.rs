use pgml_components::component;
use pgml_components::Component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "slider/template.html")]
pub struct Slider {
    cards: Vec<Component>,
    default_index: usize,
}

impl Slider {
    pub fn new() -> Slider {
        Slider {
            cards: Vec::new(),
            default_index: 0,
        }
    }

    pub fn cards(mut self, cards: Vec<Component>) -> Self {
        self.cards = cards;
        self
    }

    pub fn default_index(mut self, default_index: usize) -> Self {
        self.default_index = default_index;
        self
    }
}

component!(Slider);
