use sailfish::TemplateOnce;
use crate::components::component;
use crate::components::component::Component;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "navigation/anchor/template.html")]
pub struct Anchor {
    value: Component,
    href: String,
    data_attributes: Vec<(String, String)>,
    classes: String,
    role: String,
    pub disabled: bool,
    active: bool,
}

impl Anchor {
    pub fn new(value: Component, href: impl ToString) -> Anchor {
        Anchor {
            value,
            href: href.to_string(),
            data_attributes: vec![],
            classes: String::new(),
            role: "link".to_string(),
            disabled: false,
            active: false,
        }
    }

    pub fn stretch(self) -> Self {
        self.class("stretched-link")
    }

    pub fn class(mut self, class_name: impl ToString) -> Self {
        let mut classes = self.classes.split(" ").map(|s| s.to_string()).collect::<Vec<String>>();
        classes.push(class_name.to_string());
        self.classes = classes.join(" ");
        self
    }

    pub fn data(mut self, name: impl ToString, value: impl ToString) -> Self {
        self.data_attributes.push((name.to_string(), value.to_string()));
        self
    }

    pub fn role(mut self, role: impl ToString) -> Self {
        self.role = role.to_string();
        self
    }

    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self.class("disabled")
    }

    pub fn active(mut self) -> Self {
        self.active = true;
        self
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn value(&self) -> Component {
        self.value.clone()
    }
}

component!(Anchor);
