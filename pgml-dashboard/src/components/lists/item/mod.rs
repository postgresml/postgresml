use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "lists/item/template.html")]
pub struct Item {
    value: String,
}

impl Item {
    pub fn new() -> Item {
        Item {
            value: String::from("Your list item"),
        }
    }

    pub fn value(mut self, value: &str) -> Item {
        self.value = value.into();
        self
    }
}

component!(Item);
