use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "layouts/product/extend_head/template.html")]
pub struct ExtendHead {
    value: String,
}

impl ExtendHead {
    pub fn new() -> ExtendHead {
        ExtendHead {
            value: String::from("src/components/layouts/product/extend_head"),
        }
    }
}

component!(ExtendHead);
