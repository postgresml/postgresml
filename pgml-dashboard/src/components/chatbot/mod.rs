use pgml_components::component;
use sailfish::TemplateOnce;

// const EXAMPLE_QUESTIONS: [(&'static str, &'static str); 4] = [
//     ("Here is a Sample Question", "sample question continued"),
//     ("Here is a Sample Question", "sample question continued"),
//     ("Here is a Sample Question", "sample question continued"),
//     ("Here is a Sample Question", "sample question continued"),
// ];

type ExampleQuestions = [(&'static str, [(&'static str, &'static str); 4]); 4];
const EXAMPLE_QUESTIONS: ExampleQuestions = [
    ("PostgresML", [
        ("PostgresML", "sample question continued"),
        ("PostgresML", "sample question continued"),
        ("PostgresML", "sample question continued"),
        ("PostgresML", "sample question continued"),
    ]),
    ("PyTorch", [
        ("PyTorch", "sample question continued"),
        ("PyTorch", "sample question continued"),
        ("PyTorch", "sample question continued"),
        ("PyTorch", "sample question continued"),
    ]),
    ("Rust", [
        ("Rust", "sample question continued"),
        ("Rust", "sample question continued"),
        ("Rust", "sample question continued"),
        ("Rust", "sample question continued"),
    ]),
    ("PostgreSQL", [
        ("PostgreSQL", "sample question continued"),
        ("PostgreSQL", "sample question continued"),
        ("PostgreSQL", "sample question continued"),
        ("PostgreSQL", "sample question continued"),
    ]),
];

const KNOWLEDGE_BASES: [&'static str; 0] = [
    // "Knowledge Base 1",
    // "Knowledge Base 2",
    // "Knowledge Base 3",
    // "Knowledge Base 4",
];

const KNOWLEDGE_BASES_WITH_LOGO: [KnowledgeBaseWithLogo; 4] = [
    KnowledgeBaseWithLogo::new(
        "PostgresML",
        "/dashboard/static/images/owl_gradient.svg",
    ),
    KnowledgeBaseWithLogo::new(
        "PyTorch",
        "/dashboard/static/images/logos/pytorch.svg",
    ),
    KnowledgeBaseWithLogo::new(
        "Rust",
        "/dashboard/static/images/logos/rust.svg",
    ),
    KnowledgeBaseWithLogo::new(
        "PostgreSQL",
        "/dashboard/static/images/logos/postgresql.svg",
    ),
];

struct KnowledgeBaseWithLogo {
    name: &'static str,
    logo: &'static str,
}

impl KnowledgeBaseWithLogo {
    const fn new(name: &'static str, logo: &'static str) -> Self {
        Self { name, logo }
    }
}

const CHATBOT_BRAINS: [ChatbotBrain; 0] = [
    // ChatbotBrain::new(
    //     "PostgresML",
    //     "Falcon 180b",
    //     "/dashboard/static/images/owl_gradient.svg",
    // ),
    // ChatbotBrain::new(
    //     "OpenAI",
    //     "ChatGPT",
    //     "/dashboard/static/images/logos/openai.webp",
    // ),
    // ChatbotBrain::new(
    //     "Anthropic",
    //     "Claude",
    //     "/dashboard/static/images/logos/anthropic.webp",
    // ),
    // ChatbotBrain::new(
    //     "Meta",
    //     "Llama2 70b",
    //     "/dashboard/static/images/logos/meta.webp",
    // ),
];

struct ChatbotBrain {
    provider: &'static str,
    model: &'static str,
    logo: &'static str,
}

// impl ChatbotBrain {
//     const fn new(provider: &'static str, model: &'static str, logo: &'static str) -> Self {
//         Self {
//             provider,
//             model,
//             logo,
//         }
//     }
// }

#[derive(TemplateOnce)]
#[template(path = "chatbot/template.html")]
pub struct Chatbot {
    brains: &'static [ChatbotBrain; 0],
    example_questions: &'static ExampleQuestions,
    knowledge_bases: &'static [&'static str; 0],
    knowledge_bases_with_logo: &'static [KnowledgeBaseWithLogo; 4],
}

impl Chatbot {
    pub fn new() -> Chatbot {
        Chatbot {
            brains: &CHATBOT_BRAINS,
            example_questions: &EXAMPLE_QUESTIONS,
            knowledge_bases: &KNOWLEDGE_BASES,
            knowledge_bases_with_logo: &KNOWLEDGE_BASES_WITH_LOGO,
        }
    }
}

component!(Chatbot);
