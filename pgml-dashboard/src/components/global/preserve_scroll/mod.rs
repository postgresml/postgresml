use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "global/preserve_scroll/template.html")]
pub struct PreserveScroll {
    value: String,
}

impl PreserveScroll {
    pub fn new() -> PreserveScroll {
        PreserveScroll {
            value: String::from("src/components/global/preserve_scroll"),
        }
    }
}

component!(PreserveScroll);
