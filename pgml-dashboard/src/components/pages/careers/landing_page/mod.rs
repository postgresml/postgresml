use crate::api::cms::Collection;
use crate::components::notifications::marketing::FeatureBanner;
use crate::guards::Cluster;
use crate::notifications::Notification;
use pgml_components::component;
use sailfish::TemplateOnce;

struct Position {
    title: String,
    description: Option<String>,
    tag: Option<String>,
    href: String,
}

#[derive(TemplateOnce, Default)]
#[template(path = "pages/careers/landing_page/template.html")]
pub struct LandingPage {
    feature_banner: FeatureBanner,
    positions: Vec<Position>,
}

impl LandingPage {
    pub fn new(context: &Cluster) -> LandingPage {
        LandingPage {
            feature_banner: FeatureBanner::from_notification(Notification::next_feature(Some(context))),
            positions: Vec::new(),
        }
    }

    pub async fn index(mut self, collection: &Collection) -> LandingPage {
        let urls = collection.get_all_urls();
        for url in urls {
            let file = collection.url_to_path(url.as_ref());

            let doc = crate::api::cms::Document::from_path(&file).await.unwrap();

            let tag = match doc.tags.len() {
                0 => None,
                _ => Some(doc.tags[0].clone()),
            };

            self.positions.push(Position {
                title: doc.title,
                description: doc.description,
                tag,
                href: url,
            })
        }
        self
    }
}

component!(LandingPage);
