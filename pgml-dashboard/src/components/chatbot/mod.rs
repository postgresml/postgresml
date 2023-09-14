use pgml_components::component;
use sailfish::TemplateOnce;

const EXAMPLE_QUESTIONS: [(&'static str, &'static str); 4] = [
    ("Here is a Sample Question", "sample question continued"),
    ("Here is a Sample Question", "sample question continued"),
    ("Here is a Sample Question", "sample question continued"),
    ("Here is a Sample Question", "sample question continued"),
];

const KNOWLEDGE_BASES: [&'static str; 4] = [
    "Knowledge Base 1",
    "Knowledge Base 2",
    "Knowledge Base 3",
    "Knowledge Base 4",
];

pub struct ChatbotBrain {
    provider: String,
    model: String,
    logo: String,
}

impl ChatbotBrain {
    fn new(provider: String, model: String, logo: String) -> Self {
        Self {
            provider,
            model,
            logo,
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "chatbot/template.html")]
pub struct Chatbot {
    brains: Vec<ChatbotBrain>,
    example_questions: &'static [(&'static str, &'static str); 4],
    knowledge_bases: &'static [&'static str; 4]
}

impl Chatbot {
    pub fn new() -> Chatbot {
        let brains = vec![
            ChatbotBrain::new(
                "PostgresML".to_string(),
                "Falcon 180b".to_string(),
                "/dashboard/static/images/owl_gradient.svg".to_string(),
            ),
            ChatbotBrain::new(
                "OpenAI".to_string(),
                "ChatGPT".to_string(),
                "/dashboard/static/images/logos/openai.webp".to_string(),
            ),
            ChatbotBrain::new(
                "Anthropic".to_string(),
                "Claude".to_string(),
                "/dashboard/static/images/logos/anthropic.webp".to_string(),
            ),
            ChatbotBrain::new(
                "Meta".to_string(),
                "Llama2 70b".to_string(),
                "/dashboard/static/images/logos/meta.webp".to_string(),
            ),
        ];
        Chatbot {
            brains,
            example_questions: &EXAMPLE_QUESTIONS,
            knowledge_bases: &KNOWLEDGE_BASES
        }
    }
}

component!(Chatbot);
