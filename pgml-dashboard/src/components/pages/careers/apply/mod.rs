use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "pages/careers/apply/template.html")]
pub struct Apply {
    job_title: String,
}

impl Apply {
    pub fn new() -> Apply {
        Apply {
            job_title: String::from(""),
        }
    }

    pub fn job_title(mut self, job_title: &str) -> Apply {
        self.job_title = job_title.to_owned();
        self
    }
}

component!(Apply);
