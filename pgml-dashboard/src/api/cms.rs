use std::path::{Path, PathBuf};

use comrak::{format_html_with_plugins, parse_document, Arena, ComrakPlugins};
use rocket::{http::Status, route::Route, State};
use yaml_rust::YamlLoader;

use crate::{
    guards::Cluster,
    responses::{ResponseOk, Template},
    templates::docs::*,
    utils::{config, markdown},
};

#[get("/search?<query>", rank = 1)]
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


use rocket::fs::NamedFile;
use rocket::http::uri::Origin;

#[get("/careers/.gitbook/assets/<path>", rank = 10)]
pub async fn careers_assets(path: PathBuf) -> Option<NamedFile> {
    let path = PathBuf::from(&config::docs_dir())
        .join("careers").join(".gitbook").join("assets")
        .join(path);

    NamedFile::open(path).await.ok()
}

#[get("/careers/<path..>", rank = 5)]
async fn careers_contenthandler(mut path: PathBuf, cluster: &Cluster, origin: &Origin<'_>) -> Result<ResponseOk, Status> {
    // Rocket 0.5 began stripping trailing '/' from the path
    if origin.path().ends_with("/") {
        path = path.join("/");
    }
    let root = PathBuf::from("careers/");
    let index_path = PathBuf::from(&config::docs_dir())
        .join(&root)
        .join("SUMMARY.md");
    let contents = tokio::fs::read_to_string(&index_path).await.expect(
        format!(
            "could not read table of contents markdown: {:?}",
            index_path
        )
            .as_str(),
    );
    let mdast = ::markdown::to_mdast(&contents, &::markdown::ParseOptions::default())
        .expect("could not parse table of contents markdown");
    let url = Path::new("/careers");
    let careers = markdown::parse_summary_into_nav_links(&mdast, &url)
        .expect("could not extract nav links from table of contents");
    render(
        cluster,
        &path,
        careers,
        "Careers",
        &Path::new("careers"),
        &config::docs_dir(),
    )
        .await
}
#[get("/docs/.gitbook/assets/<path>", rank = 10)]
pub async fn docs_gitbook_assets(path: PathBuf) -> Option<NamedFile> {
    let path = PathBuf::from(&config::docs_dir())
        .join("docs/.gitbook/assets/")
        .join(path);

    NamedFile::open(path).await.ok()
}

#[get("/docs/<path..>", rank = 5)]
async fn doc_handler(mut path: PathBuf, cluster: &Cluster, origin: &Origin<'_>) -> Result<ResponseOk, Status> {
    info!("path: {:?}", path);
    if origin.path().ends_with("/") {
        path = path.join("");
    }
    info!("joined path: {:?}", path);
    let root = PathBuf::from("docs/");
    let index_path = PathBuf::from(&config::docs_dir())
        .join(&root)
        .join("SUMMARY.md");
    let contents = tokio::fs::read_to_string(&index_path).await.expect(
        format!(
            "could not read table of contents markdown: {:?}",
            index_path
        )
        .as_str(),
    );
    let mdast = ::markdown::to_mdast(&contents, &::markdown::ParseOptions::default())
        .expect("could not parse table of contents markdown");
    let url = Path::new("/docs");
    let guides = markdown::parse_summary_into_nav_links(&mdast, &url)
        .expect("could not extract nav links from table of contents");
    render(
        cluster,
        &path,
        guides,
        "Docs",
        &Path::new("docs"),
        &config::docs_dir(),
    )
    .await
}

#[get("/blog/<path..>", rank = 10)]
async fn blog_handler<'a>(path: PathBuf, cluster: &Cluster) -> Result<ResponseOk, Status> {
    render(
        cluster,
        &path,
        vec![
            NavLink::new("Speeding up vector recall by 5x with HNSW")
                .href("/blog/speeding-up-vector-recall-by-5x-with-hnsw"),
            NavLink::new("How-to Improve Search Results with Machine Learning")
                .href("/blog/how-to-improve-search-results-with-machine-learning"),
            NavLink::new("pgml-chat: A command-line tool for deploying low-latency knowledge-based chatbots: Part I")
                .href("/blog/pgml-chat-a-command-line-tool-for-deploying-low-latency-knowledge-based-chatbots-part-I"),
            NavLink::new("Announcing support for AWS us-east-1 region")
                .href("/blog/announcing-support-for-aws-us-east-1-region"),
            NavLink::new("LLM based pipelines with PostgresML and dbt (data build tool)")
                .href("/blog/llm-based-pipelines-with-postgresml-and-dbt"),
            NavLink::new("How we generate JavaScript and Python SDKs from our canonical Rust SDK")
                .href("/blog/how-we-generate-javascript-and-python-sdks-from-our-canonical-rust-sdk"),
            NavLink::new("Announcing GPTQ & GGML Quantized LLM support for Huggingface Transformers")
                .href("/blog/announcing-gptq-and-ggml-quantized-llm-support-for-huggingface-transformers"),
            NavLink::new("Making Postgres 30 Percent Faster in Production")
                .href("/blog/making-postgres-30-percent-faster-in-production"),
            NavLink::new("MindsDB vs PostgresML")
                .href("/blog/mindsdb-vs-postgresml"),
            NavLink::new("Introducing PostgresML Python SDK: Build End-to-End Vector Search Applications without OpenAI and Pinecone")
                .href("/blog/introducing-postgresml-python-sdk-build-end-to-end-vector-search-applications-without-openai-and-pinecone"),
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
        &config::blogs_dir(),
    )
    .await
}

async fn render<'a>(
    cluster: &Cluster,
    path: &'a PathBuf,
    mut nav_links: Vec<NavLink>,
    nav_title: &'a str,
    folder: &'a Path,
    content: &'a str,
) -> Result<ResponseOk, Status> {
    let mut path = path
        .to_str()
        .expect("path must convert to a string")
        .to_string();
    let url = path.clone();
    info!("path: {:?} | folder: {:?}", path, folder);
    if path.ends_with("/") || path.is_empty() {
        path.push_str("README");
    }

    // Get the document content
    let path = Path::new(&content)
        .join(folder)
        .join(&(path.to_string() + ".md"));
    info!("path: {:?}", path);

    // Read to string
    let contents = match tokio::fs::read_to_string(&path).await {
        Ok(contents) => {
            info!("loading markdown file: '{:?}", path);
            contents
        }
        Err(err) => {
            warn!("Error parsing markdown file: '{:?}' {:?}", path, err);
            return Err(Status::NotFound);
        }
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

    markdown::wrap_tables(&root, &arena).unwrap();

    // MkDocs syntax support, e.g. tabs, notes, alerts, etc.
    markdown::mkdocs(&root, &arena).unwrap();

    // Style headings like we like them
    let mut plugins = ComrakPlugins::default();
    let headings = markdown::MarkdownHeadings::new();
    plugins.render.heading_adapter = Some(&headings);
    plugins.render.codefence_syntax_highlighter = Some(&markdown::SyntaxHighlighter {});

    // Render
    let mut html = vec![];
    format_html_with_plugins(root, &markdown::options(), &mut html, &plugins).unwrap();
    let html = String::from_utf8(html).unwrap();

    // Handle navigation
    for nav_link in nav_links.iter_mut() {
        nav_link.should_open(&url, &folder);
    }

    let user = if cluster.context.user.is_anonymous() {
        None
    } else {
        Some(cluster.context.user.clone())
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
        .toc_links(&toc_links)
        .footer(cluster.context.marketing_footer.to_string());

    Ok(ResponseOk(
        layout.render(crate::templates::Article { content: html }),
    ))
}

pub fn routes() -> Vec<Route> {
    routes![docs_gitbook_assets, doc_handler, blog_handler, careers_handler, careers_gitbook_assets, search]
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::markdown::{options, MarkdownHeadings, SyntaxHighlighter};

    #[test]
    fn test_syntax_highlighting() {
        let code = r#"
# Hello

```postgresql
SELECT * FROM test;
```
        "#;

        let arena = Arena::new();
        let root = parse_document(&arena, &code, &options());

        // Style headings like we like them
        let mut plugins = ComrakPlugins::default();
        let binding = MarkdownHeadings::new();
        plugins.render.heading_adapter = Some(&binding);
        plugins.render.codefence_syntax_highlighter = Some(&SyntaxHighlighter {});

        let mut html = vec![];
        format_html_with_plugins(root, &options(), &mut html, &plugins).unwrap();
        let html = String::from_utf8(html).unwrap();

        assert!(html.contains("<span class=\"syntax-highlight\">SELECT</span>"));
    }

    #[test]
    fn test_wrapping_tables() {
        let markdown = r#"
This is some markdown with a table

| Syntax      | Description |
| ----------- | ----------- |
| Header      | Title       |
| Paragraph   | Text        |

This is the end of the markdown
        "#;

        let arena = Arena::new();
        let root = parse_document(&arena, &markdown, &options());

        let plugins = ComrakPlugins::default();

        markdown::wrap_tables(&root, &arena).unwrap();

        let mut html = vec![];
        format_html_with_plugins(root, &options(), &mut html, &plugins).unwrap();
        let html = String::from_utf8(html).unwrap();

        assert!(
            html.contains(
                r#"
<div class="overflow-auto w-100">
<table>"#
            ) && html.contains(
                r#"
</table>
</div>"#
            )
        );
    }

    #[test]
    fn test_wrapping_tables_no_table() {
        let markdown = r#"
This is some markdown with no table

This is the end of the markdown
        "#;

        let arena = Arena::new();
        let root = parse_document(&arena, &markdown, &options());

        let plugins = ComrakPlugins::default();

        markdown::wrap_tables(&root, &arena).unwrap();

        let mut html = vec![];
        format_html_with_plugins(root, &options(), &mut html, &plugins).unwrap();
        let html = String::from_utf8(html).unwrap();

        assert!(
            !html.contains(r#"<div class="overflow-auto w-100">"#) || !html.contains(r#"</div>"#)
        );
    }
}
