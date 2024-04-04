use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "headings/blue/template.html")]
pub struct Blue {
    value: String,
}

impl Blue {
    pub fn new(value: impl ToString) -> Blue {
        Blue {
            value: value.to_string(),
        }
    }
}

component!(Blue);
