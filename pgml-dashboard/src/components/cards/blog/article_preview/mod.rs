use crate::api::cms::Document;
use chrono::NaiveDate;
use pgml_components::component;
use sailfish::TemplateOnce;
use std::path::PathBuf;

#[derive(Clone)]
pub struct DocMeta {
    pub description: Option<String>,
    pub author: Option<String>,
    pub author_image: Option<String>,
    pub featured: bool,
    pub date: Option<NaiveDate>,
    pub tags: Vec<String>,
    pub image: Option<String>,
    pub title: String,
    pub path: String,
}

impl DocMeta {
    pub fn from_document(doc: Document) -> DocMeta {
        DocMeta {
            description: doc.description,
            author: doc.author,
            author_image: doc.author_image,
            featured: doc.featured,
            date: doc.date,
            tags: doc.tags,
            image: doc.image,
            title: doc.title,
            path: doc.url,
        }
    }
}

#[derive(TemplateOnce)]
#[template(path = "cards/blog/article_preview/template.html")]
pub struct ArticlePreview {
    card_type: String,
    meta: DocMeta,
    search_id: Option<i64>,
    search_result_index: Option<i64>,
}

impl ArticlePreview {
    pub fn new(meta: &DocMeta, search_id: Option<i64>, search_result_index: Option<i64>) -> ArticlePreview {
        ArticlePreview {
            card_type: String::from("default"),
            meta: meta.to_owned(),
            search_id,
            search_result_index,
        }
    }

    pub fn featured(mut self) -> Self {
        self.card_type = String::from("featured");
        self
    }

    pub fn show_image(mut self) -> Self {
        self.card_type = String::from("show_image");
        self
    }

    pub fn big(mut self) -> Self {
        self.card_type = String::from("big");
        self
    }

    pub fn long(mut self) -> Self {
        self.card_type = String::from("long");
        self
    }

    pub fn card_type(mut self, card_type: &str) -> Self {
        self.card_type = card_type.to_owned();
        self
    }

    pub async fn from_path(path: &str) -> ArticlePreview {
        let doc = Document::from_path(&PathBuf::from(path)).await.unwrap();
        let meta = DocMeta::from_document(doc);
        ArticlePreview::new(&meta, None, None)
    }
}

component!(ArticlePreview);
