use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "code_block/template.html")]
pub struct CodeBlock {}

impl CodeBlock {
    pub fn new() -> CodeBlock {
        CodeBlock {}
    }
}

component!(CodeBlock);
