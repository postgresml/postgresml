use crate::components::tables::small::row::Row;
use pgml_components::component;
use pgml_components::Component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "tables/serverless_models/template.html")]
pub struct ServerlessModels {
    style_type: String,
    embedding_models: [Component; 4],
    instruct_models: [Component; 8],
    summarization_models: [Component; 1],
}

impl ServerlessModels {
    pub fn new() -> ServerlessModels {
        ServerlessModels {
            style_type: "product".to_string(),
            embedding_models: [
                Component::from(Row::new(&[
                    "intfloat/e5-small-v2".into(),
                    "33.4".into(),
                    "512".into(),
                    "384".into(),
                    "Good quality, low latency".into(),
                ])),
                Component::from(Row::new(&[
                    "mixedbread-ai/mxbai-embed-large-v1".into(),
                    "335".into(),
                    "512".into(),
                    "1024".into(),
                    "High quality, higher latency".into(),
                ])),
                Component::from(Row::new(&[
                    "Alibaba-NLP/gte-base-en-v1.5".into(),
                    "137".into(),
                    "8192".into(),
                    "768".into(),
                    "Supports up to 8,000 input tokens".into(),
                ])),
                Component::from(Row::new(&[
                    "Alibaba-NLP/gte-large-en-v1.5".into(),
                    "434".into(),
                    "8192".into(),
                    "1024".into(),
                    "Highest quality, 8,000 input tokens".into(),
                ])),
            ],
            instruct_models: [
                Component::from(Row::new(&[
                    "meta-llama/Llama-3.2-1B-Instruct".into(),
                    "1".into(),
                    "1".into(),
                    "128".into(),
                    "Lowest latency".into(),
                ])),
                Component::from(Row::new(&[
                    "meta-llama/Llama-3.2-3B-Instruct".into(),
                    "3".into(),
                    "3".into(),
                    "128".into(),
                    "Low latency".into(),
                ])),
                Component::from(Row::new(&[
                    "meta-llama/Meta-Llama-3.1-405B-Instruct".into(),
                    "405".into(),
                    "405".into(),
                    "128k".into(),
                    "Highest quality".into(),
                ])),
                Component::from(Row::new(&[
                    "meta-llama/Meta-Llama-3.1-70B-Instruct".into(),
                    "70".into(),
                    "70".into(),
                    "128k".into(),
                    "High quality".into(),
                ])),
                Component::from(Row::new(&[
                    "meta-llama/Meta-Llama-3.1-8B-Instruct".into(),
                    "8".into(),
                    "8".into(),
                    "128k".into(),
                    "Low latency".into(),
                ])),
                Component::from(Row::new(&[
                    "microsoft/Phi-3-mini-128k-instruct".into(),
                    "3.8".into(),
                    "3.8".into(),
                    "128k".into(),
                    "Low latency".into(),
                ])),
                Component::from(Row::new(&[
                    "mistralai/Mixtral-8x7B-Instruct-v0.1".into(),
                    "56".into(),
                    "12.9".into(),
                    "32k".into(),
                    "MOE high quality".into(),
                ])),
                Component::from(Row::new(&[
                    "mistralai/Mistral-7B-Instruct-v0.2".into(),
                    "7".into(),
                    "7".into(),
                    "32k".into(),
                    "Low latency".into(),
                ])),
            ],
            summarization_models: [Component::from(Row::new(&[
                "google/pegasus-xsum".into(),
                "568".into(),
                "512".into(),
                "8k".into(),
            ]))],
        }
    }

    pub fn set_style_type(mut self, style_type: &str) -> Self {
        self.style_type = style_type.to_string();
        self
    }
}

#[derive(TemplateOnce, Default)]
#[template(path = "tables/serverless_models/turbotemplate.html")]
pub struct ServerlessModelsTurbo {
    comp: Component,
}

impl ServerlessModelsTurbo {
    pub fn new(comp: Component) -> ServerlessModelsTurbo {
        ServerlessModelsTurbo { comp }
    }
}

component!(ServerlessModels);
component!(ServerlessModelsTurbo);
