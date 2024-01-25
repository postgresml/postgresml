use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use std::str::FromStr;

use comrak::{format_html_with_plugins, parse_document, Arena, ComrakPlugins};
use lazy_static::lazy_static;
use markdown::mdast::Node;
use rocket::{fs::NamedFile, http::uri::Origin, route::Route, State};
use yaml_rust::YamlLoader;

use crate::{
    components::{cms::index_link::IndexLink, layouts::marketing::base::Theme, layouts::marketing::Base},
    guards::Cluster,
    responses::{ResponseOk, Template},
    templates::docs::*,
    utils::config,
};
use serde::{Deserialize, Serialize};
use std::fmt;

lazy_static! {
    static ref BLOG: Collection = Collection::new(
        "Blog",
        true,
        HashMap::from([
            ("announcing-hnsw-support-in-our-sdk", "speeding-up-vector-recall-5x-with-hnsw"),
            ("backwards-compatible-or-bust-python-inside-rust-inside-postgres/", "backwards-compatible-or-bust-python-inside-rust-inside-postgres"),
            ("data-is-living-and-relational/", "data-is-living-and-relational"),
            ("data-is-living-and-relational/", "data-is-living-and-relational"),
            ("generating-llm-embeddings-with-open-source-models-in-postgresml/", "generating-llm-embeddings-with-open-source-models-in-postgresml"),
            ("introducing-postgresml-python-sdk-build-end-to-end-vector-search-applications-without-openai-and-pinecone", "introducing-postgresml-python-sdk-build-end-to-end-vector-search-applications-without-openai-and-pin"),
            ("llm-based-pipelines-with-postgresml-and-dbt", "llm-based-pipelines-with-postgresml-and-dbt-data-build-tool"),
            ("oxidizing-machine-learning/", "oxidizing-machine-learning"),
            ("personalize-embedding-vector-search-results-with-huggingface-and-pgvector", "personalize-embedding-results-with-application-data-in-your-database"),
            ("pgml-chat-a-command-line-tool-for-deploying-low-latency-knowledge-based-chatbots-part-I", "pgml-chat-a-command-line-tool-for-deploying-low-latency-knowledge-based-chatbots-part-i"),
            ("postgres-full-text-search-is-awesome/", "postgres-full-text-search-is-awesome"),
            ("postgresml-is-8x-faster-than-python-http-microservices/", "postgresml-is-8-40x-faster-than-python-http-microservices"),
            ("postgresml-is-8x-faster-than-python-http-microservices", "postgresml-is-8-40x-faster-than-python-http-microservices"),
            ("postgresml-is-moving-to-rust-for-our-2.0-release/", "postgresml-is-moving-to-rust-for-our-2.0-release"),
            ("postgresml-raises-4.7m-to-launch-serverless-ai-application-databases-based-on-postgres/", "postgresml-raises-usd4.7m-to-launch-serverless-ai-application-databases-based-on-postgres"),
            ("postgresml-raises-4.7m-to-launch-serverless-ai-application-databases-based-on-postgres", "postgresml-raises-usd4.7m-to-launch-serverless-ai-application-databases-based-on-postgres"),
            ("scaling-postgresml-to-one-million-requests-per-second/", "scaling-postgresml-to-1-million-requests-per-second"),
            ("scaling-postgresml-to-one-million-requests-per-second", "scaling-postgresml-to-1-million-requests-per-second"),
            ("which-database-that-is-the-question/", "which-database-that-is-the-question"),
        ])
    );
    static ref CAREERS: Collection = Collection::new("Careers", true, HashMap::from([("a", "b")]));
    static ref DOCS: Collection = Collection::new(
        "Docs",
        false,
        HashMap::from([
            ("sdks/tutorials/semantic-search-using-instructor-model", "introduction/apis/client-sdks/tutorials/semantic-search-using-instructor-model"),
            ("data-storage-and-retrieval/documents", "resources/data-storage-and-retrieval/documents"),
            ("guides/setup/quick_start_with_docker", "resources/developer-docs/quick-start-with-docker"),
            ("guides/transformers/setup", "resources/developer-docs/quick-start-with-docker"),
            ("transformers/fine_tuning/", "introduction/apis/sql-extensions/pgml.tune"),
            ("guides/predictions/overview", "introduction/apis/sql-extensions/pgml.predict/"),
            ("machine-learning/supervised-learning/data-pre-processing", "introduction/apis/sql-extensions/pgml.train/data-pre-processing"),
        ])
    );
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum DocType {
    Blog,
    Docs,
    Careers,
}

impl fmt::Display for DocType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DocType::Blog => write!(f, "blog"),
            DocType::Docs => write!(f, "docs"),
            DocType::Careers => write!(f, "careers"),
        }
    }
}

impl FromStr for DocType {
    type Err = ();

    fn from_str(s: &str) -> Result<DocType, Self::Err> {
        match s {
            "blog" => Ok(DocType::Blog),
            "docs" => Ok(DocType::Docs),
            "careers" => Ok(DocType::Careers),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    /// The absolute path on disk
    pub path: PathBuf,
    pub description: Option<String>,
    pub author: Option<String>,
    pub author_image: Option<String>,
    pub featured: bool,
    pub date: Option<chrono::NaiveDate>,
    pub tags: Vec<String>,
    pub image: Option<String>,
    pub title: String,
    pub toc_links: Vec<TocLink>,
    pub contents: String,
    pub doc_type: Option<DocType>,
    // url to thumbnail for social share
    pub thumbnail: Option<String>,
}

// Gets document markdown
impl Document {
    pub async fn from_path(path: &PathBuf) -> anyhow::Result<Document, std::io::Error> {
        let doc_type = match path.strip_prefix(config::cms_dir()) {
            Ok(path) => match path.into_iter().next() {
                Some(dir) => match &PathBuf::from(dir).display().to_string()[..] {
                    "blog" => Some(DocType::Blog),
                    "docs" => Some(DocType::Docs),
                    "careers" => Some(DocType::Careers),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };

        if doc_type.is_none() {
            warn!("doc_type not parsed from path: {path:?}");
        }

        let contents = tokio::fs::read_to_string(&path).await?;

        let parts = contents.split("---").collect::<Vec<&str>>();

        let (meta, contents) = if parts.len() > 1 {
            match YamlLoader::load_from_str(parts[1]) {
                Ok(meta) => {
                    if meta.len() == 0 || meta[0].as_hash().is_none() {
                        (None, contents)
                    } else {
                        (Some(meta[0].clone()), parts[2..].join("---").to_string())
                    }
                }
                Err(_) => (None, contents),
            }
        } else {
            (None, contents)
        };

        let default_image_path = BLOG
            .asset_url_root
            .join("blog_image_placeholder.png")
            .display()
            .to_string();

        // parse meta section
        let (description, image, featured, tags) = match meta {
            Some(meta) => {
                let description = if meta["description"].is_badvalue() {
                    None
                } else {
                    Some(meta["description"].as_str().unwrap().to_string())
                };

                // For now the only images shown are blog images TODO: use doc_type to set asset path when working.
                let image = if meta["image"].is_badvalue() {
                    Some(default_image_path.clone())
                } else {
                    match PathBuf::from_str(meta["image"].as_str().unwrap()) {
                        Ok(image_path) => match image_path.file_name() {
                            Some(file_name) => {
                                let file = PathBuf::from(file_name).display().to_string();
                                Some(BLOG.asset_url_root.join(file).display().to_string())
                            }
                            _ => Some(default_image_path.clone()),
                        },
                        _ => Some(default_image_path.clone()),
                    }
                };

                let featured = if meta["featured"].is_badvalue() {
                    false
                } else {
                    meta["featured"].as_bool().unwrap()
                };

                let tags = if meta["tags"].is_badvalue() {
                    Vec::new()
                } else {
                    let mut tags = Vec::new();
                    for tag in meta["tags"].as_vec().unwrap() {
                        tags.push(tag.as_str().unwrap_or_else(|| "").to_string());
                    }
                    tags
                };

                (description, image, featured, tags)
            }
            None => (None, Some(default_image_path.clone()), false, Vec::new()),
        };

        let thumbnail = match &image {
            Some(image) => {
                if image.contains(&default_image_path) || doc_type != Some(DocType::Blog) {
                    None
                } else {
                    Some(format!("{}{}", config::site_domain(), image))
                }
            }
            None => None,
        };

        // Parse Markdown
        let arena = Arena::new();
        let root = parse_document(&arena, &contents, &crate::utils::markdown::options());
        let title = crate::utils::markdown::get_title(root).unwrap();
        let toc_links = crate::utils::markdown::get_toc(root).unwrap();
        let (author, date, author_image) = crate::utils::markdown::get_author(root);

        let document = Document {
            path: path.to_owned(),
            description,
            author,
            author_image,
            date,
            featured,
            tags,
            image,
            title,
            toc_links,
            contents,
            doc_type,
            thumbnail,
        };
        Ok(document)
    }

    pub fn html(self) -> String {
        let contents = self.contents;

        // Parse Markdown
        let arena = Arena::new();
        let spaced_contents = crate::utils::markdown::gitbook_preprocess(&contents);
        let root = parse_document(&arena, &spaced_contents, &crate::utils::markdown::options());

        // MkDocs, gitbook syntax support, e.g. tabs, notes, alerts, etc.
        crate::utils::markdown::mkdocs(root, &arena).unwrap();

        // Style headings like we like them
        let mut plugins = ComrakPlugins::default();
        let headings = crate::utils::markdown::MarkdownHeadings::new();
        plugins.render.heading_adapter = Some(&headings);
        plugins.render.codefence_syntax_highlighter = Some(&crate::utils::markdown::SyntaxHighlighter {});

        let mut html = vec![];
        format_html_with_plugins(root, &crate::utils::markdown::options(), &mut html, &plugins).unwrap();
        let html = String::from_utf8(html).unwrap();

        html
    }
}

/// A Gitbook collection of documents
#[derive(Default)]
pub struct Collection {
    /// The properly capitalized identifier for this collection
    name: String,
    /// The root location on disk for this collection
    pub root_dir: PathBuf,
    /// The root location for gitbook assets
    pub asset_dir: PathBuf,
    /// The base url for this collection
    url_root: PathBuf,
    /// A hierarchical list of content in this collection
    pub index: Vec<IndexLink>,
    /// A list of old paths to new paths in this collection
    redirects: HashMap<&'static str, &'static str>,
    /// Url to assets for this collection
    pub asset_url_root: PathBuf,
}

impl Collection {
    pub fn new(name: &str, hide_root: bool, redirects: HashMap<&'static str, &'static str>) -> Collection {
        info!("Loading collection: {name}");
        let name = name.to_owned();
        let slug = name.to_lowercase();
        let root_dir = config::cms_dir().join(&slug);
        let asset_dir = root_dir.join(".gitbook").join("assets");
        let url_root = PathBuf::from("/").join(&slug);
        let asset_url_root = PathBuf::from("/").join(&slug).join(".gitbook").join("assets");

        let mut collection = Collection {
            name,
            root_dir,
            asset_dir,
            url_root,
            redirects,
            asset_url_root,
            ..Default::default()
        };
        collection.build_index(hide_root);
        collection
    }

    pub async fn get_asset(&self, path: &str) -> Option<NamedFile> {
        info!("get_asset: {} {path}", self.name);

        NamedFile::open(self.asset_dir.join(path)).await.ok()
    }

    pub async fn get_content(
        &self,
        mut path: PathBuf,
        cluster: &Cluster,
        origin: &Origin<'_>,
    ) -> Result<ResponseOk, crate::responses::NotFound> {
        info!("get_content: {} | {path:?}", self.name);

        let mut redirected = false;
        match self
            .redirects
            .get(path.as_os_str().to_str().expect("needs to be a well formed path"))
        {
            Some(redirect) => {
                warn!("found redirect: {} <- {:?}", redirect, path);
                redirected = true; // reserved for some fallback path
                path = PathBuf::from(redirect);
            }
            None => {}
        };

        let canonical = format!(
            "https://postgresml.org{}/{}",
            self.url_root.to_string_lossy(),
            path.to_string_lossy()
        );
        if origin.path().ends_with("/") && !redirected {
            path = path.join("README");
        }
        let path = self.root_dir.join(format!("{}.md", path.to_string_lossy()));

        self.render(&path, &canonical, cluster).await
    }

    /// Create an index of the Collection based on the SUMMARY.md from Gitbook.
    /// Summary provides document ordering rather than raw filesystem access,
    /// in addition to formatted titles and paths.
    fn build_index(&mut self, hide_root: bool) {
        let summary_path = self.root_dir.join("SUMMARY.md");
        let summary_contents = std::fs::read_to_string(&summary_path)
            .unwrap_or_else(|_| panic!("Could not read summary: {summary_path:?}"));
        let mdast = markdown::to_mdast(&summary_contents, &::markdown::ParseOptions::default())
            .unwrap_or_else(|_| panic!("Could not parse summary: {summary_path:?}"));

        let mut index = Vec::new();
        for node in mdast
            .children()
            .unwrap_or_else(|| panic!("Summary has no content: {summary_path:?}"))
            .iter()
        {
            match node {
                Node::List(list) => {
                    let mut links = self
                        .get_sub_links(list)
                        .unwrap_or_else(|_| panic!("Could not parse list of index links: {summary_path:?}"));
                    index.append(&mut links);
                }
                _ => {
                    warn!("Irrelevant content ignored in: {summary_path:?}")
                }
            }
        }
        self.index = index;

        if self.index.is_empty() {
            error!("Index has no entries for Collection: {}", self.name);
        }

        if hide_root {
            self.index = self.index[1..].to_vec();
        }
    }

    pub fn get_sub_links(&self, list: &markdown::mdast::List) -> anyhow::Result<Vec<IndexLink>> {
        let mut links = Vec::new();

        // SUMMARY.md is a nested List > ListItem > List | Paragraph > Link > Text
        for node in list.children.iter() {
            match node {
                Node::ListItem(list_item) => {
                    for node in list_item.children.iter() {
                        match node {
                            Node::List(list) => {
                                let mut link: IndexLink = links.pop().unwrap();
                                link.children = self.get_sub_links(list).unwrap();
                                links.push(link);
                            }
                            Node::Paragraph(paragraph) => {
                                for node in paragraph.children.iter() {
                                    match node {
                                        Node::Link(link) => {
                                            for node in link.children.iter() {
                                                match node {
                                                    Node::Text(text) => {
                                                        let mut url = Path::new(&link.url)
                                                            .with_extension("")
                                                            .to_string_lossy()
                                                            .to_string();
                                                        if url.ends_with("README") {
                                                            url = url.replace("README", "");
                                                        }
                                                        let url = self.url_root.join(url);
                                                        let parent = IndexLink::new(text.value.as_str())
                                                            .href(&url.to_string_lossy());
                                                        links.push(parent);
                                                    }
                                                    _ => error!("unhandled link child: {node:?}"),
                                                }
                                            }
                                        }
                                        _ => error!("unhandled paragraph child: {node:?}"),
                                    }
                                }
                            }
                            _ => error!("unhandled list_item child: {node:?}"),
                        }
                    }
                }
                _ => error!("unhandled list child: {node:?}"),
            }
        }
        Ok(links)
    }

    // Convert a IndexLink from summary to a file path.
    pub fn url_to_path(&self, url: &str) -> PathBuf {
        let url = if url.ends_with('/') {
            format!("{url}README.md")
        } else {
            format!("{url}.md")
        };

        let mut path = PathBuf::from(url);
        if path.has_root() {
            path = path.strip_prefix("/").unwrap().to_owned();
        }

        let mut path_v = path.components().collect::<Vec<_>>();
        path_v.remove(0);

        let path_pb = PathBuf::from_iter(path_v.iter());

        self.root_dir.join(path_pb)
    }

    // get all urls in the collection and preserve order.
    pub fn get_all_urls(&self) -> Vec<String> {
        let mut urls: Vec<String> = Vec::new();
        let mut children: Vec<&IndexLink> = Vec::new();
        for item in &self.index {
            children.push(item);
        }

        children.reverse();

        while children.len() > 0 {
            let current = children.pop().unwrap();
            urls.push(current.href.clone());

            for i in (0..current.children.len()).rev() {
                children.push(&current.children[i])
            }
        }

        urls
    }

    // Sets specified index as currently viewed.
    fn open_index(&self, path: &PathBuf) -> Vec<IndexLink> {
        self.index
            .clone()
            .iter_mut()
            .map(|nav_link| {
                let mut nav_link = nav_link.clone();
                nav_link.should_open(&path);
                nav_link
            })
            .collect()
    }

    // renders document in layout
    async fn render<'a>(
        &self,
        path: &'a PathBuf,
        canonical: &str,
        cluster: &Cluster,
    ) -> Result<ResponseOk, crate::responses::NotFound> {
        let user = if cluster.context.user.is_anonymous() {
            None
        } else {
            Some(cluster.context.user.clone())
        };

        match Document::from_path(&path).await {
            Ok(doc) => {
                let index = self.open_index(&doc.path);

                let mut layout = crate::templates::Layout::new(&doc.title, Some(cluster));
                if let Some(image) = &doc.thumbnail {
                    layout.image(&image);
                }
                if let Some(description) = &doc.description {
                    layout.description(description);
                }
                if let Some(user) = &user {
                    layout.user(user);
                }

                let layout = layout
                    .canonical(canonical)
                    .nav_title(&self.name)
                    .nav_links(&index)
                    .toc_links(&doc.toc_links)
                    .footer(cluster.context.marketing_footer.to_string());

                Ok(ResponseOk(
                    layout.render(crate::templates::Article { content: doc.html() }),
                ))
            }
            // Return page not found on bad path
            _ => {
                let mut layout = crate::templates::Layout::new("404", Some(cluster));

                let doc = String::from(
                    r#"
                <div style='height: 80vh'>
                    <h2>Oops, document not found!</h2>
                    <p>The document you are searching for may have been moved or replaced with better content.</p>
                </div>"#,
                );

                if let Some(user) = &user {
                    layout.user(user);
                }

                layout
                    .nav_links(&self.index)
                    .nav_title(&self.name)
                    .footer(cluster.context.marketing_footer.to_string());

                layout.render(crate::templates::Article { content: doc });

                Err(crate::responses::NotFound(layout.into()))
            }
        }
    }
}

#[get("/search?<query>", rank = 20)]
async fn search(query: &str, index: &State<crate::utils::markdown::SearchIndex>) -> ResponseOk {
    let results = index.search(query).unwrap();

    ResponseOk(
        Template(Search {
            query: query.to_string(),
            results,
        })
        .into(),
    )
}

#[get("/blog/.gitbook/assets/<path>", rank = 10)]
pub async fn get_blog_asset(path: &str) -> Option<NamedFile> {
    BLOG.get_asset(path).await
}

#[get("/careers/.gitbook/assets/<path>", rank = 10)]
pub async fn get_careers_asset(path: &str) -> Option<NamedFile> {
    CAREERS.get_asset(path).await
}

#[get("/docs/.gitbook/assets/<path>", rank = 10)]
pub async fn get_docs_asset(path: &str) -> Option<NamedFile> {
    DOCS.get_asset(path).await
}

#[get("/blog/<path..>", rank = 5)]
async fn get_blog(
    path: PathBuf,
    cluster: &Cluster,
    origin: &Origin<'_>,
) -> Result<ResponseOk, crate::responses::NotFound> {
    BLOG.get_content(path, cluster, origin).await
}

#[get("/careers/<path..>", rank = 5)]
async fn get_careers(
    path: PathBuf,
    cluster: &Cluster,
    origin: &Origin<'_>,
) -> Result<ResponseOk, crate::responses::NotFound> {
    CAREERS.get_content(path, cluster, origin).await
}

#[get("/docs/<path..>", rank = 5)]
async fn get_docs(
    path: PathBuf,
    cluster: &Cluster,
    origin: &Origin<'_>,
) -> Result<ResponseOk, crate::responses::NotFound> {
    DOCS.get_content(path, cluster, origin).await
}

#[get("/blog")]
async fn blog_landing_page(cluster: &Cluster) -> Result<ResponseOk, crate::responses::NotFound> {
    let layout = Base::new(
        "PostgresML blog landing page, home of technical tutorials, general updates and all things AI/ML.",
        Some(cluster),
    )
    .theme(Theme::Docs)
    .footer(cluster.context.marketing_footer.to_string());

    Ok(ResponseOk(
        layout.render(
            crate::components::pages::blog::LandingPage::new(cluster)
                .index(&BLOG)
                .await,
        ),
    ))
}

#[get("/user_guides/<path..>", rank = 5)]
async fn get_user_guides(
    path: PathBuf,
    cluster: &Cluster,
    origin: &Origin<'_>,
) -> Result<ResponseOk, crate::responses::NotFound> {
    DOCS.get_content(path, cluster, origin).await
}

pub fn routes() -> Vec<Route> {
    routes![
        blog_landing_page,
        get_blog,
        get_blog_asset,
        get_careers,
        get_careers_asset,
        get_docs,
        get_docs_asset,
        get_user_guides,
        search
    ]
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::markdown::{options, MarkdownHeadings, SyntaxHighlighter};
    use regex::Regex;
    use rocket::http::{ContentType, Cookie, Status};
    use rocket::local::asynchronous::Client;
    use rocket::{Build, Rocket};

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
        let root = parse_document(&arena, markdown, &options());

        let plugins = ComrakPlugins::default();

        crate::utils::markdown::wrap_tables(root, &arena).unwrap();

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
        let root = parse_document(&arena, markdown, &options());

        let plugins = ComrakPlugins::default();

        crate::utils::markdown::wrap_tables(root, &arena).unwrap();

        let mut html = vec![];
        format_html_with_plugins(root, &options(), &mut html, &plugins).unwrap();
        let html = String::from_utf8(html).unwrap();

        assert!(!html.contains(r#"<div class="overflow-auto w-100">"#) || !html.contains(r#"</div>"#));
    }

    async fn rocket() -> Rocket<Build> {
        dotenv::dotenv().ok();
        rocket::build()
            .manage(crate::utils::markdown::SearchIndex::open().unwrap())
            .mount("/", crate::api::cms::routes())
    }

    fn gitbook_test(html: String) -> Option<String> {
        // all gitbook expresions should be removed, this catches {%  %} nonsupported expressions.
        let re = Regex::new(r"[{][%][^{]*[%][}]").unwrap();
        let rsp = re.find(&html);
        if rsp.is_some() {
            return Some(rsp.unwrap().as_str().to_string());
        }

        // gitbook TeX block not supported yet
        let re = Regex::new(r"(\$\$).*(\$\$)").unwrap();
        let rsp = re.find(&html);
        if rsp.is_some() {
            return Some(rsp.unwrap().as_str().to_string());
        }

        None
    }

    // Ensure blogs render and there are no unparsed gitbook components.
    #[sqlx::test]
    async fn render_blogs_test() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let blog: Collection = Collection::new("Blog", true, HashMap::new());

        for path in blog.index {
            let req = client.get(path.clone().href);
            let rsp = req.dispatch().await;
            let body = rsp.into_string().await.unwrap();

            let test = gitbook_test(body);

            assert!(
                test.is_none(),
                "bad html parse in {:?}. This feature is not supported {:?}",
                path.href,
                test.unwrap()
            )
        }
    }

    // Ensure Docs render and ther are no unparsed gitbook compnents.
    #[sqlx::test]
    async fn render_guides_test() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let docs: Collection = Collection::new("Docs", true, HashMap::new());

        for path in docs.index {
            let req = client.get(path.clone().href);
            let rsp = req.dispatch().await;
            let body = rsp.into_string().await.unwrap();

            let test = gitbook_test(body);

            assert!(
                test.is_none(),
                "bad html parse in {:?}. This feature is not supported {:?}",
                path.href,
                test.unwrap()
            )
        }
    }

    #[sqlx::test]
    async fn doc_not_found() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let req = client.get("/docs/not_a_doc");
        let rsp = req.dispatch().await;

        assert!(rsp.status() == Status::NotFound, "Returned status {:?}", rsp.status());
    }

    // Test backend for line highlights and line numbers added
    #[test]
    fn gitbook_codeblock_test() {
        let contents = r#"
{% code title="Test name for html" lineNumbers="true" %}
```javascript-highlightGreen="1"
    import something
    let a = 1
```
{% endcode %}
"#;

        let expected = r#"
<div class="code-block with-title line-numbers">
    <div class="title">
        Test name for html
    </div>
    <pre data-controller="copy">
        <div class="code-toolbar">
            <span data-action="click->copy#codeCopy" class="material-symbols-outlined btn-code-toolbar">content_copy</span>
            <span class="material-symbols-outlined btn-code-toolbar" disabled>link</span>
            <span class="material-symbols-outlined btn-code-toolbar" disabled>edit</span>
        </div>
        <code language='javascript' data-controller="code-block">
            <div class="highlight code-line-highlight-green">importsomething</div>
            <div class="highlight code-line-highlight-none">leta=1</div>
            <div class="highlight code-line-highlight-none"></div>
        </code>
    </pre>          
</div>"#;

        // Parse Markdown
        let arena = Arena::new();
        let spaced_contents = crate::utils::markdown::gitbook_preprocess(contents);
        let root = parse_document(&arena, &spaced_contents, &crate::utils::markdown::options());

        crate::utils::markdown::wrap_tables(root, &arena).unwrap();

        // MkDocs, gitbook syntax support, e.g. tabs, notes, alerts, etc.
        crate::utils::markdown::mkdocs(root, &arena).unwrap();

        // Style headings like we like them
        let mut plugins = ComrakPlugins::default();
        let headings = crate::utils::markdown::MarkdownHeadings::new();
        plugins.render.heading_adapter = Some(&headings);
        plugins.render.codefence_syntax_highlighter = Some(&crate::utils::markdown::SyntaxHighlighter {});

        let mut html = vec![];
        format_html_with_plugins(root, &crate::utils::markdown::options(), &mut html, &plugins).unwrap();
        let html = String::from_utf8(html).unwrap();

        println!("expected: {}", expected);

        println!("response: {}", html);

        assert!(
            html.chars().filter(|c| !c.is_whitespace()).collect::<String>()
                == expected.chars().filter(|c| !c.is_whitespace()).collect::<String>()
        )
    }

    // Test we can parse doc meta with out issue.
    #[sqlx::test]
    async fn docs_meta_parse() {
        let collection = &crate::api::cms::DOCS;

        let urls = collection.get_all_urls();

        for url in urls {
            let path = collection.url_to_path(url.as_ref());
            crate::api::cms::Document::from_path(&path).await.unwrap();
        }
    }

    // Test we can parse blog meta with out issue.
    #[sqlx::test]
    async fn blog_meta_parse() {
        let collection = &crate::api::cms::BLOG;

        let urls = collection.get_all_urls();

        for url in urls {
            let path = collection.url_to_path(url.as_ref());
            crate::api::cms::Document::from_path(&path).await.unwrap();
        }
    }

    // Test we can parse career meta with out issue.
    #[sqlx::test]
    async fn career_meta_parse() {
        let collection = &crate::api::cms::CAREERS;

        let urls = collection.get_all_urls();

        for url in urls {
            let path = collection.url_to_path(url.as_ref());
            crate::api::cms::Document::from_path(&path).await.unwrap();
        }
    }
}
