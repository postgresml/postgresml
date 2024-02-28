use crate::components::cards::blog::article_preview::ArticlePreview;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "sections/related_articles/template.html")]
pub struct RelatedArticles {
    articles: Vec<ArticlePreview>,
}

impl RelatedArticles {
    pub fn new() -> RelatedArticles {
        RelatedArticles { articles: Vec::new() }
    }

    pub fn add_article(mut self, article: ArticlePreview) -> Self {
        self.articles.push(article);
        self
    }
}

component!(RelatedArticles);
