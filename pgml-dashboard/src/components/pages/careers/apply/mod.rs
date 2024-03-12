use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "pages/careers/apply/template.html")]
pub struct Apply {
    job_title: String,
    success: Option<bool>,
}

impl Apply {
    pub fn new() -> Apply {
        Apply {
            job_title: String::from(""),
            success: None,
        }
    }

    pub fn job_title(mut self, job_title: &str) -> Apply {
        self.job_title = job_title.to_owned();
        self
    }

    pub fn success(mut self, success: bool) -> Apply {
        self.success = Some(success);
        self
    }
}

component!(Apply);
