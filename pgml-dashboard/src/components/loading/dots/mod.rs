use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "loading/dots/template.html")]
pub struct Dots {}

impl Dots {
    pub fn new() -> Dots {
        Dots {}
    }
}

component!(Dots);
