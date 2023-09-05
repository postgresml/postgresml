use crate::components::component;
use crate::components::tables::large::Row;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "tables/large/table/template.html")]
pub struct Table {
    rows: Vec<Row>,
    headers: Vec<String>,
}

impl Table {
    pub fn new(headers: &[impl ToString], rows: &[Row]) -> Table {
        Table {
            headers: headers.iter().map(|h| h.to_string()).collect(),
            rows: rows.to_vec(),
        }
    }
}

component!(Table);
