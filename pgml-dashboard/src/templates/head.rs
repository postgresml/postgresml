#[derive(Clone, Default)]
pub struct Head {
    pub title: String,
    pub description: Option<String>,
    pub image: Option<String>,
}

impl Head
{
    pub fn new() -> Head {
        Head::default()
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
