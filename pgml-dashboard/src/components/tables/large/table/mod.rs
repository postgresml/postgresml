use crate::components::tables::large::Row;
use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "tables/large/table/template.html")]
pub struct Table {
    rows: Vec<Row>,
    headers: Vec<String>,
    classes: String,
    footers: Vec<Component>,
}

impl Table {
    pub fn new(headers: &[impl ToString], rows: &[Row]) -> Table {
        Table {
            headers: headers.iter().map(|h| h.to_string()).collect(),
            rows: rows.to_vec(),
            classes: "table table-lg".to_string(),
            footers: Vec::new(),
        }
    }

    pub fn selectable(mut self) -> Self {
        self.classes.push_str(" selectable");
        self.rows = self.rows.into_iter().map(|r| r.selectable()).collect();
        self
    }

    pub fn footers(mut self, footer: Vec<Component>) -> Self {
        self.footers = footer;
        self
    }

    pub fn alt_style(mut self) -> Self {
        self.classes.push_str(" alt-style");
        self
    }
}

component!(Table);
