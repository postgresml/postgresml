use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "pages/docs/landing_page/template.html")]
pub struct LandingPage {}

impl LandingPage {
    pub fn new() -> LandingPage {
        LandingPage {}
    }
}

component!(LandingPage);
