use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "code_block/template.html")]
pub struct CodeBlock {
    content: String,
    language: String,
    editable: bool,
    id: String,
}

impl CodeBlock {
    pub fn new(content: &str) -> CodeBlock {
        CodeBlock {
            content: content.to_string(),
            language: "sql".to_string(),
            editable: false,
            id: "code-block".to_string(),
        }
    }

    pub fn set_language(mut self, language: &str) -> Self {
        self.language = language.to_owned();
        self
    }

    pub fn set_editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    pub fn set_id(mut self, id: &str) -> Self {
        self.id = id.to_owned();
        self
    }
}

component!(CodeBlock);
