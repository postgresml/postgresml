use sailfish::TemplateOnce;

#[derive(Clone, Default)]
pub struct Head {
    pub title: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub preloads: Vec<String>,
}

impl Head {
    pub fn new() -> Head {
        Head::default()
    }

    pub fn add_preload(&mut self, preload: &str) -> &mut Self {
        self.preloads.push(preload.to_owned());
        self
    }

    pub fn title(mut self, title: &str) -> Head {
        self.title = title.to_owned();
        self
    }

    pub fn description(mut self, description: &str) -> Head {
        self.description = Some(description.to_owned());
        self
    }

    pub fn image(mut self, image: &str) -> Head {
        self.image = Some(image.to_owned());
        self
    }

    pub fn not_found() -> Head {
        Head::new().title("404 - Not Found")
    }
}

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "layout/head.html")]
pub struct DefaultHeadTemplate {
    pub head: Head,
}

impl DefaultHeadTemplate {
    pub fn new(head: Option<Head>) -> DefaultHeadTemplate {
        let head = match head {
            Some(head) => head,
            None => Head::new(),
        };

        DefaultHeadTemplate { head }
    }
}

impl From<DefaultHeadTemplate> for String {
    fn from(layout: DefaultHeadTemplate) -> String {
        layout.render_once().unwrap()
    }
}
