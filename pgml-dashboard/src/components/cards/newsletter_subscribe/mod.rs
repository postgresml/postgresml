use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "cards/newsletter_subscribe/template.html")]
pub struct NewsletterSubscribe {}

impl NewsletterSubscribe {
    pub fn new() -> NewsletterSubscribe {
        NewsletterSubscribe {}
    }
}

component!(NewsletterSubscribe);
