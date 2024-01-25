use crate::api::cms::Collection;
use crate::components::cards::blog::article_preview::DocMeta;
use crate::components::cards::blog::ArticlePreview;
use crate::components::notifications::marketing::FeatureBanner;
use crate::guards::Cluster;
use crate::Notification;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "pages/blog/landing_page/template.html")]
pub struct LandingPage {
    feature_banner: FeatureBanner,
    index: Vec<DocMeta>,
    is_search: bool,
}

impl LandingPage {
    pub fn new(context: &Cluster) -> LandingPage {
        LandingPage {
            feature_banner: FeatureBanner::from_notification(Notification::next_feature(Some(context))),
            index: Vec::new(),
            is_search: false,
        }
    }

    pub async fn index(mut self, collection: &Collection) -> Self {
        let urls = collection.get_all_urls();

        for url in urls {
            let file = collection.url_to_path(url.as_ref());

            let doc = crate::api::cms::Document::from_path(&file).await.unwrap();

            let meta = DocMeta {
                description: doc.description,
                author: doc.author,
                author_image: doc.author_image,
                date: doc.date,
                image: doc.image,
                featured: doc.featured,
                tags: doc.tags,
                title: doc.title,
                path: url,
            };

            self.index.push(meta)
        }
        self
    }

    pub fn pattern(mut index: Vec<DocMeta>, is_search: bool) -> Vec<String> {
        let mut cycle = 0;
        let mut html: Vec<String> = Vec::new();

        // blogs are in cms Summary order, make the first post the big card and second long card.
        let big_index = index.remove(0);
        let long_index = index.remove(0);
        let small_image_index = index.remove(0);
        index.insert(1, long_index);
        index.insert(2, big_index);
        index.insert(6, small_image_index);

        let (layout, repeat) = if is_search {
            (
                Vec::from([
                    Vec::from(["default", "show_image", "default"]),
                    Vec::from(["default", "default", "default"]),
                    Vec::from(["show_image", "default", "default"]),
                    Vec::from(["default", "default", "default"]),
                ]),
                2,
            )
        } else {
            (
                Vec::from([
                    Vec::from(["default", "long"]),
                    Vec::from(["big", "default", "default"]),
                    Vec::from(["default", "show_image", "default"]),
                    Vec::from(["default", "default", "default"]),
                    Vec::from(["long", "default"]),
                    Vec::from(["default", "default", "default"]),
                    Vec::from(["default", "long"]),
                    Vec::from(["default", "default", "default"]),
                ]),
                4,
            )
        };

        index.reverse();
        while index.len() > 0 {
            // Get the row pattern or repeat the last two row patterns.
            let pattern = match layout.get(cycle) {
                Some(pattern) => pattern,
                _ => {
                    let a = cycle - layout.len() + repeat;
                    &layout[layout.len() - repeat + (a % repeat)]
                }
            };

            // if there is enough items to complete the row pattern make the row otherwise just add default cards.
            if index.len() > pattern.len() {
                let mut row = Vec::new();
                for _ in 0..pattern.len() {
                    row.push(index.pop())
                }

                if pattern[0] != "big" {
                    for (i, doc) in row.into_iter().enumerate() {
                        let template = pattern[i];
                        html.push(
                            ArticlePreview::new(&doc.unwrap())
                                .card_type(template)
                                .render_once()
                                .unwrap(),
                        )
                    }
                } else {
                    html.push(format!(
                        r#"
                        <div class="d-xxl-flex d-none gap-3 flex-row">
                        {}
                        <div class="d-flex flex-column gap-3">
                          {}
                          {}
                        </div>
                      </div>

                      <div class="d-xxl-none">
                        {}
                      </div>
                      <div class="d-xxl-none">
                        {}
                      </div>
                      <div class="d-xxl-none">
                        {}
                      </div>
                        "#,
                        ArticlePreview::new(&row[0].clone().unwrap())
                            .big()
                            .render_once()
                            .unwrap(),
                        ArticlePreview::new(&row[1].clone().unwrap()).render_once().unwrap(),
                        ArticlePreview::new(&row[2].clone().unwrap()).render_once().unwrap(),
                        ArticlePreview::new(&row[0].clone().unwrap()).render_once().unwrap(),
                        ArticlePreview::new(&row[1].clone().unwrap()).render_once().unwrap(),
                        ArticlePreview::new(&row[2].clone().unwrap()).render_once().unwrap()
                    ))
                }
            } else {
                html.push(
                    ArticlePreview::new(&index.pop().unwrap())
                        .card_type("default")
                        .render_once()
                        .unwrap(),
                )
            }
            cycle += 1;
        }

        html
    }
}

component!(LandingPage);
