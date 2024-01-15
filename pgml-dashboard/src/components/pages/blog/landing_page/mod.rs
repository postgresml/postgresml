use sailfish::TemplateOnce;
use pgml_components::component;
use crate::components::notifications::marketing::FeatureBanner;
use crate::guards::Cluster;
use crate::Notification;
use crate::components::cms::index_link::IndexLink;
use std::path::PathBuf;
use crate::api::cms::Collection;

pub struct DocMeta {
    pub description: Option<String>,
    pub author: Option<String>,
    pub author_image: Option<String>,
    pub featured: bool, 
    pub date: Option<String>,
    pub tags: Vec<String>,
    pub image: Option<String>,
    pub title: String,
    pub path: String,
}

#[derive(TemplateOnce, Default)]
#[template(path = "pages/blog/landing_page/template.html")]
pub struct LandingPage {
    feature_banner: FeatureBanner,
    index: Vec<DocMeta>,
}

impl LandingPage {
    pub fn new(context: &Cluster) -> LandingPage {
        LandingPage {
            feature_banner: FeatureBanner::from_notification(Notification::next_feature(Some(
                context,
            ))),
            index: Vec::new(),
        }
    }

    pub async fn index(mut self, collection: &Collection) -> Self {
        let index = &collection.index;

        for item in index {
            let path = &item.href.replace("/blog/", "");
            let root = collection.root_dir.clone();
            let file = root.join(format!("{}.md", path));

            let (description, author, date, image, featured, tags, title, _, author_image) = crate::api::cms::Document::meta(&PathBuf::from(file)).await;

            println!("image: {:?}", author_image);

            let meta = DocMeta {
                description, 
                author,
                author_image,
                date,
                image,
                featured,
                tags,
                title,
                path: item.href.clone(),
            };
            // println!("item {:?}", meta);
            self.index.push(meta)
        }
        self
    }
}

component!(LandingPage);