use crate::components::cards::blog::article_preview::{ArticlePreview, DocMeta};
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "pages/blog/blog_search/response/template.html")]
pub struct Response {
    html: Vec<String>,
}

impl Response {
    pub fn new() -> Response {
        Response { html: Vec::new() }
    }

    pub fn pattern(mut self, mut articles: Vec<DocMeta>, is_search: bool) -> Response {
        let mut cycle = 0;
        let mut html: Vec<String> = Vec::new();

        let (layout, repeat) = if is_search {
            (
                Vec::from([
                    Vec::from(["default", "default", "default"]),
                    Vec::from(["default", "default", "default"]),
                    Vec::from(["default", "default", "default"]),
                    Vec::from(["default", "default", "default"]),
                ]),
                2,
            )
        } else {
            // Apply special layout if the user did not specify a query.
            // Blogs are in cms Summary order, make the first post the big card and second long card.
            let big_index = articles.remove(0);
            let long_index = articles.remove(0);
            let small_image_index = articles.remove(0);
            articles.insert(1, long_index);
            articles.insert(2, big_index);
            articles.insert(6, small_image_index);

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

        articles.reverse();
        while articles.len() > 0 {
            // Get the row pattern or repeat the last two row patterns.
            let pattern = match layout.get(cycle) {
                Some(pattern) => pattern,
                _ => {
                    let a = cycle - layout.len() + repeat;
                    &layout[layout.len() - repeat + (a % repeat)]
                }
            };

            // if there is enough items to complete the row pattern make the row otherwise just add default cards.
            if articles.len() > pattern.len() {
                let mut row = Vec::new();
                for _ in 0..pattern.len() {
                    row.push(articles.pop())
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
                    ArticlePreview::new(&articles.pop().unwrap())
                        .card_type("default")
                        .render_once()
                        .unwrap(),
                )
            }
            cycle += 1;
        }

        self.html = html;
        self
    }
}

component!(Response);
