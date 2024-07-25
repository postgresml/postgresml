use crate::components::cards::blog::article_preview::DocMeta;
use crate::components::notifications::marketing::FeatureBanner;
use crate::guards::Cluster;
use crate::notifications::Notification;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "pages/blog/landing_page/template.html")]
pub struct LandingPage {
    feature_banner: FeatureBanner,
    featured_cards: Vec<DocMeta>,
}

impl LandingPage {
    pub fn new(context: &Cluster) -> LandingPage {
        LandingPage {
            feature_banner: FeatureBanner::from_notification(Notification::next_feature(Some(context))),
            featured_cards: Vec::new(),
        }
    }

    pub fn featured_cards(mut self, docs: Vec<DocMeta>) -> Self {
        self.featured_cards = docs;
        self
    }
}

component!(LandingPage);
