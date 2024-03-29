use crate::components::cards::marketing::Slider as SliderCard;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "slider/template.html")]
pub struct Slider {
    cards: Vec<SliderCard>,
}

impl Slider {
    pub fn new() -> Slider {
        Slider { cards: Vec::new() }
    }

    pub fn cards(mut self, cards: Vec<SliderCard>) -> Self {
        self.cards = cards;
        self
    }
}

component!(Slider);
