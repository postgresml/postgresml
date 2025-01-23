use crate::components::cms::IndexLink;
use crate::components::layouts::Head;
use crate::guards::Cluster;
use crate::models::User;
use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "layouts/docs/template.html")]
pub struct Docs {
    head: Head,
    footer: Option<String>,
    user: Option<User>,
    content: Option<String>,
    index: Vec<IndexLink>,
    body_components: Vec<Component>,
}

impl Docs {
    pub fn new(title: &str, context: Option<&Cluster>) -> Docs {
        let (head, footer, user, body_components) = match context.as_ref() {
            Some(context) => (
                Head::new().title(&title).context(&context.context.head_items),
                Some(context.context.marketing_footer.clone()),
                Some(context.context.user.clone()),
                context.context.body_components.clone(),
            ),
            None => (Head::new().title(&title), None, None, Vec::new()),
        };

        Docs {
            head,
            footer,
            user,
            body_components,
            ..Default::default()
        }
    }

    pub fn index(mut self, index: &Vec<IndexLink>) -> Docs {
        self.index = index.clone();
        self
    }

    pub fn image(mut self, image: &Option<String>) -> Docs {
        if let Some(image) = image {
            self.head = self.head.image(image.as_str());
        }
        self
    }

    pub fn canonical(mut self, canonical: &str) -> Docs {
        self.head = self.head.canonical(canonical);
        self
    }

    pub fn render<T>(mut self, template: T) -> String
    where
        T: sailfish::TemplateOnce,
    {
        self.content = Some(template.render_once().unwrap());
        self.clone().into()
    }
}

impl From<Docs> for String {
    fn from(layout: Docs) -> String {
        layout.render_once().unwrap()
    }
}

component!(Docs);
