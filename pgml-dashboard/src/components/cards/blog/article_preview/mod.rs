use chrono::NaiveDate;
use pgml_components::component;
use sailfish::TemplateOnce;

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

#[derive(TemplateOnce)]
#[template(path = "cards/blog/article_preview/template.html")]
pub struct ArticlePreview {
    card_type: String,
    meta: DocMeta,
}

impl ArticlePreview {
    pub fn new(meta: &DocMeta) -> ArticlePreview {
        ArticlePreview {
            card_type: String::from("default"),
            meta: meta.to_owned(),
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
}

component!(ArticlePreview);
