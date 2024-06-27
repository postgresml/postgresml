use crate::components::static_nav_link::StaticNavLink;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "sections/footers/marketing_footer/template.html")]
pub struct MarketingFooter {
    product: Vec<StaticNavLink>,
    solutions: Vec<StaticNavLink>,
    resources: Vec<StaticNavLink>,
    company: Vec<StaticNavLink>,
}

impl MarketingFooter {
    pub fn new() -> MarketingFooter {
        MarketingFooter {
            product: vec![
                StaticNavLink::new("Korvus".into(), "https://github.com/postgresml/korvus".into()),
                StaticNavLink::new("PGML".into(), "https://github.com/postgresml/postgresml".into()),
                StaticNavLink::new("PpCat Learning".into(), "https://github.com/postgresml/pgcat".into()),
                StaticNavLink::new("PostgresML".into(), "/docs/product/cloud-database/".into()),
                StaticNavLink::new("VPC".into(), "/docs/product/vpn_key".into()).disabled(true),
            ],
            solutions: vec![
                StaticNavLink::new("NLP".into(), "/docs/guides/natural-language-processing".into()),
                StaticNavLink::new("Supervised Learning".into(), "/docs/guides/supervised-learning".into()),
                StaticNavLink::new("Embedding".into(), "/docs/api/sql-extension/pgml.embed".into()),
                StaticNavLink::new("Vector Database".into(), "/docs/product/vector-database".into()),
                StaticNavLink::new(
                    "Search".into(),
                    "/docs/guides/improve-search-results-with-machine-learning".into(),
                ),
                StaticNavLink::new("Chatbot".into(), "/chatbot".into()),
            ],
            resources: vec![
                StaticNavLink::new("Documentation".into(), "/docs/".into()),
                StaticNavLink::new("Blog".into(), "/blog".into()),
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

    pub fn product(mut self, product: Vec<StaticNavLink>) -> MarketingFooter {
        self.product = product;
        self
    }
}

component!(MarketingFooter);
