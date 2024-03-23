use crate::api::cms::{Document, DOCS};
use crate::components::cms::IndexLink;
use crate::components::notifications::marketing::FeatureBanner;
use crate::guards::Cluster;
use crate::Notification;
use lazy_static::lazy_static;
use pgml_components::component;
use sailfish::TemplateOnce;
use std::collections::HashMap;

lazy_static! {
    static ref ICON_MAP: HashMap<String, String> = HashMap::from([
        ("pgml.embed()", "view_array"),
        ("pgml.transform()", "transform"),
        ("pgml.tune()", "manufacturing"),
        ("pgml.train()", "model_training"),
        ("pgml.deploy()", "deployed_code"),
        ("pgml.predict()", "account_tree"),
        ("installation", "fullscreen"),
        ("collections", "overview_key"),
        ("pipelines", "climate_mini_split"),
        ("semantic search", "book"),
        ("semantic search using instructor model", "book"),
        ("postgresml is 8-40x faster than python http microservices", "fit_page"),
        ("scaling to 1 million requests per second", "bolt"),
        ("mindsdb vs postgresml", "arrow_split"),
        ("ggml quantized llm support for huggingface transformers", "transform"),
        ("making postres 30% faster in production", "30fps_select"),
    ])
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v.to_owned()))
    .collect();
    static ref AI_TARGETS: Vec<String> = Vec::from(["pgml.embed()", "pgml.transform()", "pgml.tune()"])
        .into_iter()
        .map(|s| s.to_owned())
        .collect();
    static ref ML_TARGETS: Vec<String> = Vec::from(["pgml.train()", "pgml.deploy()", "pgml.predict()"])
        .into_iter()
        .map(|s| s.to_owned())
        .collect();
    static ref OVERVIEW_TARGETS: Vec<String> = Vec::from(["installation", "collections", "pipelines"])
        .into_iter()
        .map(|s| s.to_owned())
        .collect();
    static ref TUTORIAL_TARGETS: Vec<String> =
        Vec::from(["semantic search", "semantic search using instructor model",])
            .into_iter()
            .map(|s| s.to_owned())
            .collect();
    static ref BENCHMARKS_TARGETS: Vec<String> = Vec::from([
        "postgresml is 8-40x faster than python http microservices",
        "scaling to 1 million requests per second",
        "mindsdb vs postgresml",
        "ggml quantized llm support for huggingface transformers",
        "making postgres 30 percent faster in production"
    ])
    .into_iter()
    .map(|s| s.to_owned())
    .collect();
}

#[derive(TemplateOnce, Default)]
#[template(path = "pages/docs/landing_page/template.html")]
pub struct LandingPage {
    sql_extensions_ai: Vec<DocCard>,
    sql_extensions_ml: Vec<DocCard>,
    benchmarks: Vec<DocCard>,
    client_sdks_overview: Vec<DocCard>,
    client_sdks_tutorials: Vec<DocCard>,
    feature_banner: FeatureBanner,
}

impl LandingPage {
    pub fn new(context: &Cluster) -> LandingPage {
        LandingPage {
            feature_banner: FeatureBanner::from_notification(Notification::next_feature(Some(context))),
            ..Default::default()
        }
    }

    pub async fn parse_sections(mut self, links: Vec<IndexLink>) -> Self {
        let mut children: Vec<IndexLink> = links.clone();

        let mut benchmarks_folder: Vec<IndexLink> = Vec::new();
        let mut extension_folder: Vec<IndexLink> = Vec::new();
        let mut client_sdks_folder: Vec<IndexLink> = Vec::new();
        while !children.is_empty() {
            let link = children.pop().unwrap();

            match link.title.to_lowercase().as_ref() {
                "benchmarks" => benchmarks_folder = link.children,
                "sql extension" => extension_folder = link.children,
                "client sdk" => client_sdks_folder = link.children,
                _ => {
                    if !link.children.is_empty() {
                        for item in link.children.clone() {
                            children.push(item.clone())
                        }
                    }
                }
            }
        }

        let find_targets = |links: Vec<IndexLink>, targets: &Vec<String>| -> Vec<IndexLink> {
            let mut children: Vec<IndexLink> = links.clone();
            let mut out: Vec<IndexLink> = Vec::new();

            while !children.is_empty() {
                let link = children.pop().unwrap();

                if targets.contains(&link.title.to_lowercase()) {
                    out.push(link.clone());
                }

                if !link.children.is_empty() {
                    for item in link.children.clone() {
                        children.push(item.clone())
                    }
                }
            }

            out
        };

        let benchmarks = find_targets(benchmarks_folder, &BENCHMARKS_TARGETS);
        let client_sdks_overview = find_targets(client_sdks_folder.clone(), &OVERVIEW_TARGETS);
        let client_sdks_tutorials = find_targets(client_sdks_folder, &TUTORIAL_TARGETS);
        let sql_extensions_ai = find_targets(extension_folder.clone(), &AI_TARGETS);
        let sql_extensions_ml = find_targets(extension_folder, &ML_TARGETS);

        for item in benchmarks {
            let card = DocCard::from_index_link(&item).await;
            self.benchmarks.push(card);
        }

        for item in client_sdks_overview {
            let card = DocCard::from_index_link(&item).await;
            self.client_sdks_overview.push(card);
        }

        for item in client_sdks_tutorials {
            let card = DocCard::from_index_link(&item).await;
            self.client_sdks_tutorials.push(card);
        }

        for item in sql_extensions_ai {
            let card = DocCard::from_index_link(&item).await;
            self.sql_extensions_ai.push(card);
        }

        for item in sql_extensions_ml {
            let card = DocCard::from_index_link(&item).await;
            self.sql_extensions_ml.push(card);
        }

        self
    }
}

#[derive(TemplateOnce, Default)]
#[template(path = "pages/docs/landing_page/card_template.html")]
pub struct DocCard {
    icon: String,
    title: String,
    description: String,
    icon_color: String,
    href: String,
}

impl DocCard {
    pub fn new() -> DocCard {
        DocCard {
            icon_color: String::new(),
            ..Default::default()
        }
    }

    pub async fn from_index_link(index: &IndexLink) -> DocCard {
        let path = DOCS.url_to_path(&index.href);
        let doc = Document::from_path(&path).await.unwrap();

        let title = index.title.to_lowercase();

        let icon_color = if AI_TARGETS.contains(&title) || ML_TARGETS.contains(&title) {
            "text-gradient-orange"
        } else if OVERVIEW_TARGETS.contains(&title) || TUTORIAL_TARGETS.contains(&title) {
            "text-gradient-blue"
        } else {
            "text-gradient-green"
        };

        DocCard {
            icon: ICON_MAP
                .get(&index.title.to_lowercase())
                .unwrap_or(&"book".to_owned())
                .to_owned(),
            title: index.title.clone(),
            description: doc.description.clone().unwrap_or_else(|| "No description".to_owned()),
            icon_color: icon_color.to_owned(),
            href: index.href.clone(),
        }
    }
}

#[derive(TemplateOnce, Default)]
#[template(path = "pages/docs/landing_page/alt_card_template.html")]
struct AltDocCard {
    icon: String,
    title: String,
    href: String,
}

impl AltDocCard {
    pub fn new() -> AltDocCard {
        AltDocCard {
            icon: String::new(),
            title: String::new(),
            href: String::new(),
        }
    }

    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = icon.to_owned();
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_owned();
        self
    }

    pub fn href(mut self, href: &str) -> Self {
        self.href = href.to_owned();
        self
    }
}

component!(LandingPage);
