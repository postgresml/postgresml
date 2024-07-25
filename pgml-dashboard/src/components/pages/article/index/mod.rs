use crate::api::cms::DocType;
use crate::api::cms::Document;
use crate::api::cms::BLOG;
use crate::components::cards::blog::ArticlePreview;
use crate::components::notifications::marketing::FeatureBanner;
use crate::components::sections::related_articles::RelatedArticles;
use crate::guards::Cluster;
use crate::notifications::Notification;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "pages/article/index/template.html")]
pub struct Index {
    doc: Document,
    feature_banner: FeatureBanner,
    article_type: DocType,
    document_not_found: bool,
    related_articles: RelatedArticles,
}

impl Index {
    pub fn new(context: &Cluster) -> Index {
        Index {
            feature_banner: FeatureBanner::from_notification(Notification::next_feature(Some(context))),
            doc: Document::new(),
            article_type: DocType::Blog,
            document_not_found: false,
            related_articles: RelatedArticles::new(),
        }
    }

    pub async fn document(mut self, doc: Document) -> Index {
        // for now the related articles are hardcoded
        let related_articles = RelatedArticles::new()
            .add_article(
                ArticlePreview::from_path(
                    &BLOG
                        .url_to_path("/blog/generating-llm-embeddings-with-open-source-models-in-postgresml")
                        .display()
                        .to_string(),
                )
                .await,
            )
            .add_article(
                ArticlePreview::from_path(
                    &BLOG
                        .url_to_path("/blog/making-postgres-30-percent-faster-in-production")
                        .display()
                        .to_string(),
                )
                .await,
            )
            .add_article(
                ArticlePreview::from_path(
                    &BLOG
                        .url_to_path(
                            "/blog/introducing-the-openai-switch-kit-move-from-closed-to-open-source-ai-in-minutes",
                        )
                        .display()
                        .to_string(),
                )
                .await,
            );

        self.doc = doc;
        self.related_articles = related_articles;
        self
    }

    pub fn is_blog(mut self) -> Index {
        self.article_type = DocType::Blog;
        self
    }

    pub fn is_careers(mut self) -> Index {
        self.article_type = DocType::Careers;
        self
    }

    pub fn document_not_found(mut self) -> Index {
        self.document_not_found = true;
        self
    }
}

component!(Index);
