use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "tables/small/table/template.html")]
pub struct Table {
    classes: String,
    headers: Vec<String>,
    rows: Vec<Component>,
    footers: Vec<Component>,
}

impl Table {
    pub fn new(headers: &[impl ToString], rows: &[Component]) -> Table {
        Table {
            headers: headers.iter().map(|h| h.to_string()).collect(),
            classes: "table table-sm".into(),
            rows: rows.to_vec(),
            footers: vec![],
        }
    }

    pub fn footers(mut self, footer: Vec<Component>) -> Self {
        self.footers = footer;
        self
    }
}

component!(Table);
