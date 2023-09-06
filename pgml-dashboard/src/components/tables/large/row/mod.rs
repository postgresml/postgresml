use crate::components::component;
use crate::components::component::Component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "tables/large/row/template.html")]
pub struct Row {
    columns: Vec<Component>,
    action: String,
}

impl Row {
    pub fn new(columns: &[Component]) -> Row {
        Row {
            columns: columns.to_vec(),
            action: "click->tables-large-table#selectRow".to_string(),
        }
    }

    pub fn action(mut self, action: &str) -> Self {
        self.action.push_str(&format!(" {}", action));
        self
    }
}

component!(Row);
