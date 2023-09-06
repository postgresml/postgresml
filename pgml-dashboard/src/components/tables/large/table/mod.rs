use crate::components::tables::large::Row;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "tables/large/table/template.html")]
pub struct Table {
    rows: Vec<Row>,
    headers: Vec<String>,
    classes: String,
}

impl Table {
    pub fn new(headers: &[impl ToString], rows: &[Row]) -> Table {
        Table {
            headers: headers.iter().map(|h| h.to_string()).collect(),
            rows: rows.to_vec(),
            classes: "table table-lg".to_string(),
        }
    }

    pub fn selectable(mut self) -> Self {
        self.classes.push_str(" selectable");
        self
    }
}

component!(Table);
