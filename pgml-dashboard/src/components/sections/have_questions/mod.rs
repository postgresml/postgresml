use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "sections/have_questions/template.html")]
pub struct HaveQuestions {}

impl HaveQuestions {
    pub fn new() -> HaveQuestions {
        HaveQuestions {}
    }
}

component!(HaveQuestions);
