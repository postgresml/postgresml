use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "layouts/head/template.html")]
pub struct Head {
    pub title: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub preloads: Vec<String>,
    pub context: Option<String>,
    pub canonical: Option<String>,
}

impl Head {
    pub fn new() -> Head {
        Head::default()
    }

    pub fn add_preload(mut self, preload: &str) -> Head {
        self.preloads.push(preload.to_owned());
        self
    }

    pub fn title(mut self, title: &str) -> Head {
        self.title = title.to_owned();
        self
    }

    pub fn description(mut self, description: &str) -> Head {
        self.description = if description.len() == 0 {
            None
        } else {
            Some(description.to_owned())
        };
        self
    }

    pub fn canonical(mut self, canonical: &str) -> Head {
        self.canonical = if canonical.len() == 0 {
            None
        } else {
            Some(canonical.to_owned())
        };
        self
    }

    pub fn image(mut self, image: &str) -> Head {
        self.image = if image.len() == 0 { None } else { Some(image.to_owned()) };
        self
    }

    pub fn not_found() -> Head {
        Head::new().title("404 - Not Found")
    }

    pub fn context(mut self, context: &Option<String>) -> Head {
        self.context = context.to_owned();
        self
    }
}

component!(Head);

#[cfg(test)]
mod head_tests {
    use super::Head;

    #[test]
    fn new_head() {
        let head = Head::new();
        assert_eq!(
            (head.title, head.description, head.image, head.preloads),
            ("".to_string(), None, None, vec![])
        );
    }

    // #[test]
    // fn add_preload() {
    //     let mut head = Head::new();
    //     let mut preloads: Vec<String> = vec![];
    //     for i in 0..5 {
    //         preloads.push(format!("image/test_preload_{}.test", i).to_string());
    //     }
    //     for preload in preloads.clone() {
    //         head.add_preload(&preload);
    //     }
    //     assert!(head.preloads.eq(&preloads));
    // }

    #[test]
    fn add_title() {
        let head = Head::new().title("test title");
        assert_eq!(head.title, "test title");
    }

    #[test]
    fn add_description() {
        let head = Head::new().description("test description");
        assert_eq!(head.description, Some("test description".to_string()));
    }

    #[test]
    fn add_image() {
        let head = Head::new().image("images/image_file_path.jpg");
        assert_eq!(head.image, Some("images/image_file_path.jpg".to_string()));
    }

    #[test]
    fn not_found() {
        let head = Head::not_found();
        assert_eq!(head.title, "404 - Not Found")
    }
}

#[cfg(test)]
mod default_head_template_test {
    use super::Head;
    use sailfish::TemplateOnce;

    #[test]
    fn default() {
        let head = Head::new();
        let rendered = head.render_once().unwrap();

        assert!(
            rendered.contains(r#"<head>"#) &&
            rendered.contains(r#"<title> – PostgresML</title>"#) &&
            rendered.contains(r#"<meta name="description" content="Train and deploy models to make online predictions using only SQL, with an open source Postgres extension.">"#) &&
            !rendered.contains("preload") &&
            rendered.contains("</head>")
        )
    }

    #[test]
    fn set_head() {
        let mut head = Head::new()
            .title("test title")
            .description("test description")
            .image("image/test_image.jpg");
        // head.add_preload("image/test_preload.webp");

        let rendered = head.render_once().unwrap();
        assert!(
            rendered.contains("<title>test title – PostgresML</title>") &&
            rendered.contains(r#"<meta name="description" content="test description">"#) &&
            rendered.contains(r#"<meta property="og:image" content="image/test_image.jpg">"#) && 
            !rendered.contains(r#"<link rel="preload" fetchpriority="high" as="image" href="image/test_preload.webp" type="image/webp">"#)
        );
    }
}
