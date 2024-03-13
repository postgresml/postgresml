use crate::models;
use crate::utils::config;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/navbar/marketing/template.html")]
pub struct Marketing {
    pub current_user: Option<models::User>,
    pub standalone_dashboard: bool,
    pub style_alt: bool,
    pub no_transparent_nav: bool,
}

impl Marketing {
    pub fn new(user: Option<models::User>) -> Marketing {
        Marketing {
            current_user: user,
            standalone_dashboard: config::standalone_dashboard(),
            style_alt: false,
            no_transparent_nav: false,
        }
    }

    pub fn style_alt(mut self) -> Self {
        self.style_alt = true;
        self
    }

    pub fn no_transparent_nav(mut self, no_transparent_nav: bool) -> Self {
        self.no_transparent_nav = no_transparent_nav;
        self
    }
}

component!(Marketing);
