use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "postgres_logo/template.html")]
pub struct PostgresLogo {
    link: String,
    bigger: bool,
}

impl PostgresLogo {
    pub fn new(link: &str) -> PostgresLogo {
        PostgresLogo {
            link: link.to_owned(),
            bigger: false,
        }
    }

    pub fn bigger(mut self) -> PostgresLogo {
        self.bigger = true;
        self
    }
}

component!(PostgresLogo);
