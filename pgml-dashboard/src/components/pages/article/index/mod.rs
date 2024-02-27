use crate::api::cms::DocType;
use crate::api::cms::Document;
use crate::components::notifications::marketing::FeatureBanner;
use crate::guards::Cluster;
use crate::Notification;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "pages/article/index/template.html")]
pub struct Index {
    doc: Document,
    feature_banner: FeatureBanner,
    article_type: DocType,
    document_not_found: bool,
}

impl Index {
    pub fn new(context: &Cluster) -> Index {
        Index {
            feature_banner: FeatureBanner::from_notification(Notification::next_feature(Some(context))),
            doc: Document::new(),
            article_type: DocType::Blog,
            document_not_found: false,
        }
    }

    pub fn document(mut self, doc: Document) -> Index {
        self.doc = doc;
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
