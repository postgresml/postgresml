use serde::Deserialize;

#[derive(FromForm)]
pub struct Notebook<'a> {
    pub name: &'a str,
}

#[derive(FromForm)]
pub struct Cell<'a> {
    pub contents: &'a str,
    pub cell_type: &'a str,
}

#[derive(FromForm)]
pub struct Upload<'a> {
    pub file: TempFile<'a>,
    pub has_header: bool,
}

#[derive(Deserialize)]
pub struct Reorder {
    pub cells: Vec<i64>,
}

#[derive(Deserialize)]
pub struct ChatbotPostData {
    pub question: String,
    pub model: u8,
    #[serde(rename = "knowledgeBase")]
    pub knowledge_base: u8,
}
