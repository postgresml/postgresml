use pgml_components::component;
use pgml_components::Component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "tables/large/row/template.html")]
pub struct Row {
    columns: Vec<Component>,
    action: String,
    data: Vec<(String, String)>,
}

impl Row {
    pub fn new(columns: &[Component]) -> Row {
        Row {
            columns: columns.to_vec(),
            action: "".to_string(),
            data: vec![],
        }
    }

    pub fn action(mut self, action: &str) -> Self {
        self.action.push_str(&format!(" {}", action));
        self
    }

    pub fn data(mut self, name: &str, value: &str) -> Self {
        self.data.push((name.to_owned(), value.to_owned()));
        self
    }

    pub fn selectable(self) -> Self {
        self.action("click->tables-large-table#selectRow")
    }
}

component!(Row);
