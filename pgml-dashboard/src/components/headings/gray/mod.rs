use sailfish::TemplateOnce;
use pgml_components::component;

#[derive(TemplateOnce, Default)]
#[template(path = "headings/gray/template.html")]
pub struct Gray {
    value: String,
}

impl Gray {
    pub fn new(value: impl ToString) -> Gray {
        Gray {
            value: value.to_string(),
        }
    }
}

component!(Gray);
