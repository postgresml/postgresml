use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "tables/small/row/template.html")]
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
