use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "buttons/goto_btn/template.html")]
pub struct GotoBtn {
    href: String,
    text: String,
}

impl GotoBtn {
    pub fn new() -> GotoBtn {
        GotoBtn {
            href: String::new(),
            text: String::new(),
        }
    }

    pub fn set_href(mut self, href: &str) -> Self {
        self.href = href.into();
        self
    }

    pub fn set_text(mut self, text: &str) -> Self {
        self.text = text.into();
        self
    }
}

component!(GotoBtn);
