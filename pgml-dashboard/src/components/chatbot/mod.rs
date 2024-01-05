use pgml_components::component;
use sailfish::TemplateOnce;

type ExampleQuestions = [(&'static str, [(&'static str, &'static str); 4]); 4];
const EXAMPLE_QUESTIONS: ExampleQuestions = [
    (
        "postgresml",
        [
            ("How do I", "use pgml.transform()?"),
            ("Show me", "a query to train a model"),
            ("What is HNSW", "indexing"),
            ("Teach me", "how to use pgml.embed()"),
        ],
    ),
    (
        "pytorch",
        [
            ("What are", "tensors?"),
            ("How do I", "train a model?"),
            ("Show me", "some features of PyTorch"),
            ("Explain", "how to use an optimizer?"),
        ],
    ),
    (
        "rust",
        [
            ("What is", "a lifetime?"),
            ("How do I", "use a for loop?"),
            ("Show me", "an example of using map"),
            ("Explain", "the borrow checker"),
        ],
    ),
    (
        "postgresql",
        [
            ("How do I", "join two tables?"),
            ("What is", "a GIN index?"),
            ("When should I", "use an outer join?"),
            ("Explain", "what relational data is"),
        ],
    ),
];

const KNOWLEDGE_BASES_WITH_LOGO: [KnowledgeBaseWithLogo; 4] = [
    KnowledgeBaseWithLogo::new(
        "postgresml",
        "PostgresML",
        "/dashboard/static/images/owl_gradient.svg",
    ),
    KnowledgeBaseWithLogo::new(
        "pytorch",
        "PyTorch",
        "/dashboard/static/images/logos/pytorch.svg",
    ),
    KnowledgeBaseWithLogo::new("rust", "Rust", "/dashboard/static/images/logos/rust.svg"),
    KnowledgeBaseWithLogo::new(
        "postgresql",
        "PostgreSQL",
        "/dashboard/static/images/logos/postgresql.svg",
    ),
];

struct KnowledgeBaseWithLogo {
    id: &'static str,
    name: &'static str,
    logo: &'static str,
}

impl KnowledgeBaseWithLogo {
    const fn new(id: &'static str, name: &'static str, logo: &'static str) -> Self {
        Self { id, name, logo }
    }
}

const CHATBOT_BRAINS: [ChatbotBrain; 1] = [
    // ChatbotBrain::new(
    //     "teknium/OpenHermes-2.5-Mistral-7B",
    //     "OpenHermes",
    //     "teknium/OpenHermes-2.5-Mistral-7B",
    //     "/dashboard/static/images/logos/openhermes.webp",
    // ),
    // ChatbotBrain::new(
    //     "Gryphe/MythoMax-L2-13b",
    //     "MythoMax",
    //     "Gryphe/MythoMax-L2-13b",
    //     "/dashboard/static/images/logos/mythomax.webp",
    // ),
    ChatbotBrain::new(
        "openai",
        "OpenAI",
        "ChatGPT",
        "/dashboard/static/images/logos/openai.webp",
    ),
    // ChatbotBrain::new(
    //     "berkeley-nest/Starling-LM-7B-alpha",
    //     "Starling",
    //     "berkeley-nest/Starling-LM-7B-alpha",
    //     "/dashboard/static/images/logos/starling.webp",
    // ),
];

struct ChatbotBrain {
    id: &'static str,
    provider: &'static str,
    model: &'static str,
    logo: &'static str,
}

impl ChatbotBrain {
    const fn new(
        id: &'static str,
        provider: &'static str,
        model: &'static str,
        logo: &'static str,
    ) -> Self {
        Self {
            id,
            provider,
            model,
            logo,
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "chatbot/template.html")]
pub struct Chatbot {
    brains: &'static [ChatbotBrain; 1],
    example_questions: &'static ExampleQuestions,
    knowledge_bases_with_logo: &'static [KnowledgeBaseWithLogo; 4],
}

impl Default for Chatbot {
    fn default() -> Self {
        Chatbot {
            brains: &CHATBOT_BRAINS,
            example_questions: &EXAMPLE_QUESTIONS,
            knowledge_bases_with_logo: &KNOWLEDGE_BASES_WITH_LOGO,
        }
    }
}

impl Chatbot {
    pub fn new() -> Self {
        Self::default()
    }
}

component!(Chatbot);
