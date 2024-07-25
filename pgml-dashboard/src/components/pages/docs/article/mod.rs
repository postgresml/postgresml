use crate::components::notifications::marketing::FeatureBanner;
use crate::guards::Cluster;
use crate::notifications::Notification;
use crate::templates::docs::TocLink;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "pages/docs/article/template.html")]
pub struct Article {
    toc_links: Vec<TocLink>,
    content: String,
    document_not_found: bool,
    feature_banner: FeatureBanner,
}

impl Article {
    pub fn new(context: &Cluster) -> Article {
        Article {
            feature_banner: FeatureBanner::from_notification(Notification::next_feature(Some(context))),
            ..Default::default()
        }
    }

    pub fn toc_links(mut self, toc_links: &Vec<TocLink>) -> Self {
        self.toc_links = toc_links.clone();
        self
    }

    pub fn content(mut self, content: &str) -> Self {
        self.content = content.to_owned();
        self
    }

    pub fn document_not_found(mut self) -> Self {
        self.document_not_found = true;
        self
    }
}

component!(Article);
