use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "cards/newsletter_subscribe/template.html")]
pub struct NewsletterSubscribe {
    success: Option<bool>,
    error_message: Option<String>,
    email: Option<String>,
}

impl NewsletterSubscribe {
    pub fn new() -> NewsletterSubscribe {
        NewsletterSubscribe {
            success: None,
            error_message: None,
            email: None,
        }
    }

    pub fn success(mut self, success: bool) -> Self {
        self.success = Some(success);
        self
    }

    pub fn error_message(mut self, error_message: &str) -> Self {
        self.error_message = Some(error_message.to_owned());
        self
    }

    pub fn email(mut self, email: &str) -> Self {
        self.email = Some(email.to_owned());
        self
    }
}

component!(NewsletterSubscribe);
