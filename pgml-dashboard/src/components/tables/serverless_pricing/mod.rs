use crate::components::tables::small::row::Row;
use pgml_components::component;
use pgml_components::Component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "tables/serverless_pricing/template.html")]
pub struct ServerlessPricing {
    style_type: String,
    pricing: [Component; 6],
}

impl ServerlessPricing {
    pub fn new() -> ServerlessPricing {
        ServerlessPricing {
            style_type: "product".to_string(),
            pricing: [
                Component::from(Row::new(&[
                    "Tables & index storage".into(),
                    "$0.25/GB per month".into(),
                ])),
                Component::from(Row::new(&[
                    "Retrieval, filtering, ranking & other queries".into(),
                    "$7.50 per hour".into(),
                ])),
                Component::from(Row::new(&["Embeddings".into(), "Included w/ queries".into()])),
                Component::from(Row::new(&["LLMs".into(), "Included w/ queries".into()])),
                Component::from(Row::new(&["Fine tuning".into(), "Included w/ queries".into()])),
                Component::from(Row::new(&["Machine learning".into(), "Included w/ queries".into()])),
            ],
        }
    }

    pub fn set_style_type(mut self, style_type: &str) -> ServerlessPricing {
        self.style_type = style_type.to_string();
        self
    }
}

#[derive(TemplateOnce, Default)]
#[template(path = "tables/serverless_pricing/turbotemplate.html")]
pub struct ServerlessPricingTurbo {
    comp: Component,
}

impl ServerlessPricingTurbo {
    pub fn new(comp: Component) -> ServerlessPricingTurbo {
        ServerlessPricingTurbo { comp }
    }
}

component!(ServerlessPricing);
component!(ServerlessPricingTurbo);
