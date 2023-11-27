use crate::components::static_nav_link::StaticNavLink;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "sections/footers/marketing_footer/template.html")]
pub struct MarketingFooter {
    solutions: Vec<StaticNavLink>,
    resources: Vec<StaticNavLink>,
    company: Vec<StaticNavLink>,
}

impl MarketingFooter {
    pub fn new() -> MarketingFooter {
        MarketingFooter {
            solutions: vec![
                StaticNavLink::new("Overview".into(), "/docs/".into()),
                StaticNavLink::new("Chatbot".into(), "/chatbot".into()),
                StaticNavLink::new("Site Search".into(), "/search".into()).disabled(true),
                StaticNavLink::new("Fraud Detection".into(), "/fraud".into()).disabled(true),
                StaticNavLink::new("Forecasting".into(), "/forecasting".into()).disabled(true),
            ],
            resources: vec![
                StaticNavLink::new("Documentation".into(), "/docs/".into()),
                StaticNavLink::new("Blog".into(), "/blog/".into()),
            ],
            company: vec![
                StaticNavLink::new("Careers".into(), "/careers/".into()),
                StaticNavLink::new("Contact".into(), "mailto:team@postgresml.org".into()),
            ],
        }
    }

    pub fn solutions(mut self, solutions: Vec<StaticNavLink>) -> MarketingFooter {
        self.solutions = solutions;
        self
    }

    pub fn resources(mut self, resources: Vec<StaticNavLink>) -> MarketingFooter {
        self.resources = resources;
        self
    }

    pub fn company(mut self, company: Vec<StaticNavLink>) -> MarketingFooter {
        self.company = company;
        self
    }
}

component!(MarketingFooter);
