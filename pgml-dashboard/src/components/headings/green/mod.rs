use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "headings/green/template.html")]
pub struct Green {
    value: String,
    style: String,
}

impl Green {
    pub fn new(value: impl ToString) -> Green {
        Green {
            value: value.to_string(),
            style: "font-size: 18px;".into(),
        }
    }
}

component!(Green);
