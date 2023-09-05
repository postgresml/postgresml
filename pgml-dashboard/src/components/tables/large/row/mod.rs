use crate::components::component;
use crate::components::component::Component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "tables/large/row/template.html")]
pub struct Row {
    columns: Vec<Component>,
}

impl Row {
    pub fn new(columns: &[Component]) -> Row {
        Row {
            columns: columns.to_vec(),
        }
    }
}

component!(Row);
