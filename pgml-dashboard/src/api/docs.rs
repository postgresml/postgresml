use std::path::{Path, PathBuf};

use comrak::{
    format_html_with_plugins, parse_document, Arena, ComrakPlugins,
};
use rocket::{http::Status, route::Route, State};
use yaml_rust::YamlLoader;

use crate::{
    guards::Cluster,
    responses::{ResponseOk, Template},
    templates::docs::*,
    utils::{config, markdown},
};


#[get("/docs/search?<query>", rank = 1)]
async fn search(query: &str, index: &State<markdown::SearchIndex>) -> ResponseOk {
    let results = index.search(query).unwrap();

    ResponseOk(
        Template(Search {
            query: query.to_string(),
            results,
        })
        .into(),
    )
}

#[get("/docs/<path..>", rank = 10)]
async fn doc_handler<'a>(path: PathBuf, cluster: Cluster) -> Result<ResponseOk, Status> {
    let guides = vec![
        NavLink::new("Setup").children(vec![
            NavLink::new("Installation").children(vec![
                NavLink::new("v2").href("/docs/guides/setup/v2/installation"),
                NavLink::new("Upgrade from v1.0 to v2.0")
                    .href("/docs/guides/setup/v2/upgrade-from-v1"),
                NavLink::new("v1").href("/docs/guides/setup/installation"),
            ]),
            NavLink::new("Quick Start with Docker")
                .href("/docs/guides/setup/quick_start_with_docker"),
            NavLink::new("Distributed Training").href("/docs/guides/setup/distributed_training"),
            NavLink::new("GPU Support").href("/docs/guides/setup/gpu_support"),
            NavLink::new("Developer Setup").href("/docs/guides/setup/developers"),
        ]),
        NavLink::new("Training").children(vec![
            NavLink::new("Overview").href("/docs/guides/training/overview"),
            NavLink::new("Algorithm Selection").href("/docs/guides/training/algorithm_selection"),
            NavLink::new("Hyperparameter Search")
                .href("/docs/guides/training/hyperparameter_search"),
            NavLink::new("Preprocessing Data").href("/docs/guides/training/preprocessing"),
            NavLink::new("Joint Optimization").href("/docs/guides/training/joint_optimization"),
        ]),
        NavLink::new("Predictions").children(vec![
            NavLink::new("Overview").href("/docs/guides/predictions/overview"),
            NavLink::new("Deployments").href("/docs/guides/predictions/deployments"),
            NavLink::new("Batch Predictions").href("/docs/guides/predictions/batch"),
        ]),
        NavLink::new("Transformers").children(vec![
            NavLink::new("Setup").href("/docs/guides/transformers/setup"),
            NavLink::new("Pre-trained Models").href("/docs/guides/transformers/pre_trained_models"),
            NavLink::new("Fine Tuning").href("/docs/guides/transformers/fine_tuning"),
            NavLink::new("Embeddings").href("/docs/guides/transformers/embeddings"),
        ]),
        NavLink::new("Vector Operations").children(vec![
            NavLink::new("Overview").href("/docs/guides/vector_operations/overview")
        ]),
        NavLink::new("Dashboard").href("/docs/guides/dashboard/overview"),
        NavLink::new("Schema").children(vec![
            NavLink::new("Models").href("/docs/guides/schema/models"),
            NavLink::new("Snapshots").href("/docs/guides/schema/snapshots"),
            NavLink::new("Projects").href("/docs/guides/schema/projects"),
            NavLink::new("Deployments").href("/docs/guides/schema/deployments"),
        ]),
    ];

    render(cluster, &path, guides, "Guides", &Path::new("docs")).await
}

#[get("/blog/<path..>", rank = 10)]
async fn blog_handler<'a>(path: PathBuf, cluster: Cluster) -> Result<ResponseOk, Status> {
    render(
        cluster,
        &path,
        vec![
            NavLink::new("PostgresML raises $4.7M to launch serverless AI application databases based on Postgres")
                .href("/blog/postgresml-raises-4.7M-to-launch-serverless-ai-application-databases-based-on-postgres"),
            NavLink::new("PG Stat Sysinfo, a Postgres Extension for Querying System Statistics")
                .href("/blog/pg-stat-sysinfo-a-pg-extension"),
            NavLink::new("PostgresML as a memory backend to Auto-GPT")
                .href("/blog/postgresml-as-a-memory-backend-to-auto-gpt"),
            NavLink::new("Personalize embedding search results with Huggingface and pgvector")
                .href(
                "/blog/personalize-embedding-vector-search-results-with-huggingface-and-pgvector",
            ),
            NavLink::new("Tuning vector recall while generating query embeddings in the database")
                .href(
                    "/blog/tuning-vector-recall-while-generating-query-embeddings-in-the-database",
                ),
            NavLink::new("Generating LLM embeddings with open source models in PostgresML")
                .href("/blog/generating-llm-embeddings-with-open-source-models-in-postgresml"),
            NavLink::new("Scaling PostgresML to 1 Million Requests per Second")
                .href("/blog/scaling-postgresml-to-one-million-requests-per-second"),
            NavLink::new("PostgresML is 8-40x faster than Python HTTP Microservices")
                .href("/blog/postgresml-is-8x-faster-than-python-http-microservices"),
            NavLink::new("Backwards Compatible or Bust: Python Inside Rust Inside Postgres")
                .href("/blog/backwards-compatible-or-bust-python-inside-rust-inside-postgres"),
            NavLink::new("PostresML is Moving to Rust for our 2.0 Release")
                .href("/blog/postgresml-is-moving-to-rust-for-our-2.0-release"),
            NavLink::new("Which Database, That is the Question")
                .href("/blog/which-database-that-is-the-question"),
            NavLink::new("Postgres Full Text Search is Awesome")
                .href("/blog/postgres-full-text-search-is-awesome"),
            NavLink::new("Oxidizing Machine Learning").href("/blog/oxidizing-machine-learning"),
            NavLink::new("Data is Living and Relational")
                .href("/blog/data-is-living-and-relational"),
        ],
        "Blog",
        &Path::new("blog"),
    )
    .await
}

async fn render<'a>(
    cluster: Cluster,
    path: &'a PathBuf,
    mut nav_links: Vec<NavLink>,
    nav_title: &'a str,
    folder: &'a Path,
) -> Result<ResponseOk, Status> {
    let url = path.clone();

    // Get the document content
    let path = Path::new(&config::content_dir())
        .join(folder)
        .join(&(path.to_str().unwrap().to_string() + ".md"));

    // Read to string
    let contents = match tokio::fs::read_to_string(&path).await {
        Ok(contents) => contents,
        Err(_) => return Err(Status::NotFound),
    };
    let parts = contents.split("---").collect::<Vec<&str>>();
    let ((image, description), contents) = if parts.len() > 1 {
        match YamlLoader::load_from_str(parts[1]) {
            Ok(meta) => {
                if !meta.is_empty() {
                    let meta = meta[0].clone();
                    if meta.as_hash().is_none() {
                        ((None, None), contents.to_string())
                    } else {
                        let description: Option<String> = match meta["description"].is_badvalue() {
                            true => None,
                            false => Some(meta["description"].as_str().unwrap().to_string()),
                        };

                        let image: Option<String> = match meta["image"].is_badvalue() {
                            true => None,
                            false => Some(meta["image"].as_str().unwrap().to_string()),
                        };

                        ((image, description), parts[2..].join("---").to_string())
                    }
                } else {
                    ((None, None), contents.to_string())
                }
            }
            Err(_) => ((None, None), contents.to_string()),
        }
    } else {
        ((None, None), contents.to_string())
    };

    // Parse Markdown
    let arena = Arena::new();
    let root = parse_document(&arena, &contents, &markdown::options());

    // Title of the document is the first (and typically only) <h1>
    let title = markdown::get_title(&root).unwrap();
    let toc_links = markdown::get_toc(&root).unwrap();

    // MkDocs syntax support, e.g. tabs, notes, alerts, etc.
    markdown::mkdocs(&root, &arena).unwrap();

    // Style headings like we like them
    let mut plugins = ComrakPlugins::default();
    plugins.render.heading_adapter = Some(&markdown::MarkdownHeadings {});
    plugins.render.codefence_syntax_highlighter = Some(&markdown::SyntaxHighlighter {});

    // Render
    let mut html = vec![];
    format_html_with_plugins(root, &markdown::options(), &mut html, &plugins).unwrap();
    let html = String::from_utf8(html).unwrap();

    // Handle navigation
    for nav_link in nav_links.iter_mut() {
        nav_link.should_open(&url.to_str().unwrap().to_string());
    }

    let user = if cluster.context.user.is_anonymous() {
        None
    } else {
        Some(cluster.context.user)
    };

    let mut layout = crate::templates::Layout::new(&title);
    if image.is_some() {
        layout.image(&image.unwrap());
    }
    if description.is_some() {
        layout.description(&description.unwrap());
    }
    if user.is_some() {
        layout.user(&user.unwrap());
    }
    let layout = layout
        .nav_title(nav_title)
        .nav_links(&nav_links)
        .toc_links(&toc_links);

    Ok(ResponseOk(
        layout.render(crate::templates::Article { content: html })
    ))
}

pub fn routes() -> Vec<Route> {
    routes![doc_handler, blog_handler, search]
}
