use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "postgres_logo/template.html")]
pub struct PostgresLogo {
    link: String,
    bigger: bool,
    hide_owl: bool,
    collapsable: bool,
}

impl PostgresLogo {
    pub fn new(link: &str) -> PostgresLogo {
        PostgresLogo {
            link: link.to_owned(),
            bigger: false,
            hide_owl: false,
            collapsable: false,
        }
    }

    pub fn bigger(mut self) -> PostgresLogo {
        self.bigger = true;
        self
    }

    pub fn hide_owl(mut self) -> PostgresLogo {
        self.hide_owl = true;
        self
    }

    pub fn collapsable(mut self) -> PostgresLogo {
        self.collapsable = true;
        self
    }
}

component!(PostgresLogo);
