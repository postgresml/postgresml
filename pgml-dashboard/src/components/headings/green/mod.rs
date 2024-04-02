use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "headings/green/template.html")]
pub struct Green {
    value: String,
}

impl Green {
    pub fn new(value: impl ToString) -> Green {
        Green {
            value: value.to_string(),
        }
    }
}

component!(Green);
