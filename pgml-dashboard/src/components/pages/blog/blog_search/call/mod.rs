use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "pages/blog/blog_search/call/template.html")]
pub struct Call {}

impl Call {
    pub fn new() -> Call {
        Call {}
    }
}

component!(Call);
