use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use rocket::response::Redirect;
use std::str::FromStr;

use comrak::{format_html_with_plugins, parse_document, Arena, ComrakPlugins};
use lazy_static::lazy_static;
use markdown::mdast::Node;
use rocket::{fs::NamedFile, http::uri::Origin, route::Route, State};
use yaml_rust::YamlLoader;

use crate::{
    components::{cms::index_link::IndexLink, layouts::marketing::base::Theme, layouts::marketing::Base},
    guards::Cluster,
    responses::{Error, Response, ResponseOk, Template},
    templates::docs::*,
    utils::{config, markdown::SearchResult},
};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::components::cards::blog::article_preview;
use sailfish::TemplateOnce;

lazy_static! {
    pub static ref BLOG: Collection = Collection::new(
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
    pub static ref DOCS: Collection = Collection::new(
        "Docs",
        false,
        HashMap::from([
            ("sdks/tutorials/semantic-search-using-instructor-model", "api/client-sdk/tutorials/semantic-search-using-instructor-model"),
            ("data-storage-and-retrieval/documents", "resources/data-storage-and-retrieval/documents"),
            ("guides/setup/quick_start_with_docker", "resources/developer-docs/quick-start-with-docker"),
            ("guides/transformers/setup", "resources/developer-docs/quick-start-with-docker"),
            ("transformers/fine_tuning/", "api/sql-extension/pgml.tune"),
            ("guides/predictions/overview", "api/sql-extension/pgml.predict/"),
            ("machine-learning/supervised-learning/data-pre-processing", "api/sql-extension/pgml.train/data-pre-processing"),
            ("api/client-sdk/getting-started", "api/client-sdk/"),
            ("introduction/getting-started/import-your-data/", "introduction/import-your-data/"),
            ("introduction/getting-started/import-your-data/foreign-data-wrapper", "introduction/import-your-data/foreign-data-wrappers"),
            ("use-cases/embeddings/generating-llm-embeddings-with-open-source-models-in-postgresml", "open-source/pgml/guides/embeddings/in-database-generation"),
            ("use-cases/natural-language-processing", "open-source/pgml/guides/natural-language-processing"),
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

#[derive(Debug, Serialize, Deserialize, Default)]
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
    pub url: String,
    pub ignore: bool,
}

// Gets document markdown
impl Document {
    pub fn new() -> Document {
        Document { ..Default::default() }
    }

    // make a document from a uri of form <blog || docs || careers>/< path and file name >
    pub async fn from_url(url: &str) -> anyhow::Result<Document, std::io::Error> {
        let doc_type = match url.split('/').collect::<Vec<&str>>().get(1) {
            Some(&"blog") => Some(DocType::Blog),
            Some(&"docs") => Some(DocType::Docs),
            Some(&"careers") => Some(DocType::Careers),
            _ => None,
        };

        let path = match doc_type {
            Some(DocType::Blog) => BLOG.url_to_path(url),
            Some(DocType::Docs) => DOCS.url_to_path(url),
            Some(DocType::Careers) => CAREERS.url_to_path(url),
            _ => PathBuf::new(),
        };

        Document::from_path(&path).await
    }

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

        let default_image_path = match doc_type {
            Some(DocType::Blog) => BLOG
                .asset_url_root
                .join("blog_image_placeholder.png")
                .display()
                .to_string(),
            _ => String::from("/dashboard/static/images/careers_article_default.png"),
        };

        // parse meta section
        let (description, image, featured, tags, ignore) = match meta {
            Some(meta) => {
                let description = if meta["description"].is_badvalue() {
                    None
                } else {
                    Some(meta["description"].as_str().unwrap().to_string())
                };

                let image = if meta["image"].is_badvalue() {
                    Some(default_image_path.clone())
                } else {
                    match PathBuf::from_str(meta["image"].as_str().unwrap()) {
                        Ok(image_path) => match image_path.file_name() {
                            Some(file_name) => {
                                let file = PathBuf::from(file_name).display().to_string();
                                match doc_type {
                                    Some(DocType::Docs) => Some(DOCS.asset_url_root.join(file).display().to_string()),
                                    Some(DocType::Careers) => {
                                        Some(CAREERS.asset_url_root.join(file).display().to_string())
                                    }
                                    _ => Some(BLOG.asset_url_root.join(file).display().to_string()),
                                }
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

                let ignore = if meta["ignore"].is_badvalue() {
                    false
                } else {
                    meta["ignore"].as_bool().unwrap_or(false)
                };

                (description, image, featured, tags, ignore)
            }
            None => (None, Some(default_image_path.clone()), false, Vec::new(), false),
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

        // convert author image relative url path to absolute url path
        let author_image = if author_image.is_some() {
            let image = author_image.clone().unwrap();
            let image = PathBuf::from(image);
            let image = image.file_name().unwrap();
            match &doc_type {
                Some(DocType::Blog) => Some(BLOG.asset_url_root.join(image.to_str().unwrap()).display().to_string()),
                Some(DocType::Docs) => Some(DOCS.asset_url_root.join(image.to_str().unwrap()).display().to_string()),
                Some(DocType::Careers) => Some(
                    CAREERS
                        .asset_url_root
                        .join(PathBuf::from(image.to_str().unwrap()))
                        .display()
                        .to_string(),
                ),
                _ => None,
            }
        } else {
            None
        };

        let url = match doc_type {
            Some(DocType::Blog) => BLOG.path_to_url(&path),
            Some(DocType::Docs) => DOCS.path_to_url(&path),
            Some(DocType::Careers) => CAREERS.path_to_url(&path),
            _ => String::new(),
        };

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
            url,
            ignore,
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
        crate::utils::markdown::wrap_tables(root, &arena).unwrap();

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

    pub fn ignore(&self) -> bool {
        self.ignore
    }
}

#[derive(Debug, Clone)]
pub struct ContentPath {
    path: PathBuf,
    canonical: String,
    redirected: bool,
}

impl ContentPath {
    /// Should we issue a 301 redirect instead.
    pub fn redirect(&self) -> bool {
        self.redirected
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn canonical(&self) -> String {
        self.canonical.clone()
    }
}

impl From<ContentPath> for PathBuf {
    fn from(path: ContentPath) -> PathBuf {
        path.path
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
        debug!("Loading collection: {name}");
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
        debug!("get_asset: {} {path}", self.name);

        NamedFile::open(self.asset_dir.join(path)).await.ok()
    }

    /// Get the actual path on disk to the content being requested.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the content being requested.
    /// * `origin` - The HTTP origin of the request.
    ///
    pub async fn get_content_path(&self, mut path: PathBuf, origin: &Origin<'_>) -> ContentPath {
        debug!("get_content: {} | {path:?}", self.name);

        match self
            .redirects
            .get(path.as_os_str().to_str().expect("needs to be a well formed path"))
        {
            Some(redirect) => {
                debug!("found redirect: {} <- {:?}", redirect, path);

                return ContentPath {
                    redirected: true,
                    path: PathBuf::from(redirect),
                    canonical: "".into(),
                };
            }
            None => (),
        }

        let canonical = format!(
            "https://postgresml.org{}/{}",
            self.url_root.to_string_lossy(),
            path.to_string_lossy()
        );

        if origin.path().ends_with("/") {
            path = path.join("README");
        }

        let path = self.root_dir.join(format!("{}.md", path.to_string_lossy()));

        let path = ContentPath {
            path,
            canonical,
            redirected: false,
        };

        path
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

        let mut parent_folder: Option<String> = None;
        let mut index = Vec::new();
        let indent_level = 1;

        // Docs gets a home link added to the index
        match self.name.as_str() {
            "Docs" => {
                index.push(IndexLink::new("Documentation", indent_level).href("/docs"));
            }
            _ => {}
        }
        for node in mdast
            .children()
            .unwrap_or_else(|| panic!("Summary has no content: {summary_path:?}"))
            .iter()
        {
            match node {
                Node::List(list) => {
                    let links: Vec<IndexLink> = self
                        .get_sub_links(list, indent_level)
                        .unwrap_or_else(|_| panic!("Could not parse list of index links: {summary_path:?}"));

                    let mut out = match parent_folder.as_ref() {
                        Some(parent_folder) => {
                            let mut parent = IndexLink::new(parent_folder.as_ref(), 0).href("");
                            parent.children = links.clone();
                            Vec::from([parent])
                        }
                        None => links,
                    };

                    index.append(&mut out);
                    parent_folder = None;
                }
                Node::Heading(heading) => {
                    if heading.depth == 2 {
                        parent_folder = Some(heading.children[0].to_string());
                    }
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

    pub fn get_sub_links(&self, list: &markdown::mdast::List, indent_level: i32) -> anyhow::Result<Vec<IndexLink>> {
        let mut links = Vec::new();

        // SUMMARY.md is a nested List > ListItem > List | Paragraph > Link > Text
        for node in list.children.iter() {
            match node {
                Node::ListItem(list_item) => {
                    for node in list_item.children.iter() {
                        match node {
                            Node::List(list) => {
                                let mut link: IndexLink = links.pop().unwrap();
                                link.children = self.get_sub_links(list, indent_level + 1).unwrap();
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
                                                        let parent = IndexLink::new(text.value.as_str(), indent_level)
                                                            .href(&url.to_string_lossy());
                                                        links.push(parent);
                                                    }
                                                    _ => warn!("unhandled link child: {node:?}"),
                                                }
                                            }
                                        }
                                        _ => warn!("unhandled paragraph child: {node:?}"),
                                    }
                                }
                            }
                            _ => warn!("unhandled list_item child: {node:?}"),
                        }
                    }
                }
                _ => warn!("unhandled list child: {node:?}"),
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

    // Convert a file path to a url
    pub fn path_to_url(&self, path: &PathBuf) -> String {
        let url = path.strip_prefix(config::cms_dir()).unwrap();
        let url = format!("/{}", url.display().to_string());

        let url = if url.ends_with("README.md") {
            url.replace("README.md", "")
        } else {
            url
        };

        let url = if url.ends_with(".md") {
            url.replace(".md", "")
        } else {
            url
        };
        url
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
            if current.href.len() > 0 {
                urls.push(current.href.clone());
            }

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
    ) -> Result<Response, crate::responses::NotFound> {
        match Document::from_path(&path).await {
            Ok(doc) => {
                let head = crate::components::layouts::Head::new()
                    .title(&doc.title)
                    .description(&doc.description.clone().unwrap_or_else(|| String::new()))
                    .image(&doc.thumbnail.clone().unwrap_or_else(|| String::new()))
                    .canonical(&canonical);

                let layout = Base::from_head(head, Some(cluster)).theme(Theme::Docs);

                let mut article = crate::components::pages::article::Index::new(&cluster)
                    .document(doc)
                    .await;

                article = if self.name == "Blog" {
                    article.is_blog()
                } else {
                    article.is_careers()
                };

                Ok(Response::ok(layout.render(article)))
            }
            // Return page not found on bad path
            _ => {
                let layout = Base::new("404", Some(cluster)).theme(Theme::Docs);

                let mut article = crate::components::pages::article::Index::new(&cluster).document_not_found();

                article = if self.name == "Blog" {
                    article.is_blog()
                } else {
                    article.is_careers()
                };

                Err(crate::responses::NotFound(layout.render(article)))
            }
        }
    }
}

#[get("/search?<query>", rank = 20)]
async fn search(query: &str, site_search: &State<crate::utils::markdown::SiteSearch>) -> ResponseOk {
    let results = site_search
        .search(query, None, None)
        .await
        .expect("Error performing search");

    let results: Vec<SearchResult> = results
        .into_iter()
        .map(|document| {
            let snippet = if let Some(description) = document.description {
                description
            } else {
                let author = document.author.unwrap_or_else(|| String::from("xzxzxz"));
                // The heuristics used here are ok, not the best it will be better when we can just use the description field
                document
                    .contents
                    .lines()
                    .find(|l| !l.is_empty() && !l.contains(&document.title) && !l.contains(&author) && l.len() > 30)
                    .unwrap_or("")
                    .split(' ')
                    .take(20)
                    .collect::<Vec<&str>>()
                    .join(" ")
                    + "&nbsp;..."
            };
            let path = document
                .path
                .to_str()
                .unwrap_or_default()
                .replace(".md", "")
                .replace(&config::static_dir().display().to_string(), "");
            SearchResult {
                title: document.title,
                path,
                snippet,
            }
        })
        .collect();

    ResponseOk(
        Template(Search {
            query: query.to_string(),
            results,
        })
        .into(),
    )
}

#[get("/search_blog?<query>&<tag>", rank = 20)]
async fn search_blog(query: &str, tag: &str, site_search: &State<crate::utils::markdown::SiteSearch>) -> ResponseOk {
    let tag = if tag.len() > 0 {
        Some(Vec::from([tag.to_string()]))
    } else {
        None
    };

    // If user is not making a search return all blogs in default design.
    let results = if query.len() > 0 || tag.clone().is_some() {
        let results = site_search.search(query, Some(DocType::Blog), tag.clone()).await;

        let results = match results {
            Ok(results) => results
                .into_iter()
                .map(|document| article_preview::DocMeta::from_document(document))
                .collect::<Vec<article_preview::DocMeta>>(),
            Err(_) => Vec::new(),
        };

        results
    } else {
        let mut results = Vec::new();

        for url in BLOG.get_all_urls() {
            let doc = Document::from_url(&url).await.unwrap();

            results.push(article_preview::DocMeta::from_document(doc));
        }

        results
    };

    let is_search = query.len() > 0 || tag.is_some();

    ResponseOk(
        crate::components::pages::blog::blog_search::Response::new()
            .pattern(results, is_search)
            .render_once()
            .unwrap(),
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
) -> Result<Response, crate::responses::NotFound> {
    let content_path = BLOG.get_content_path(path, origin).await;

    if content_path.redirect() {
        let redirect = Path::new("/blog/").join(content_path.path()).display().to_string();
        return Ok(Response::redirect(redirect));
    }

    let canonical = content_path.canonical();
    BLOG.render(&content_path.into(), &canonical, cluster).await
}

#[get("/careers/<path..>", rank = 5)]
async fn get_careers(
    path: PathBuf,
    cluster: &Cluster,
    origin: &Origin<'_>,
) -> Result<Response, crate::responses::NotFound> {
    let content_path = CAREERS.get_content_path(path, origin).await;

    if content_path.redirect() {
        let redirect = Path::new("/blog/").join(content_path.path()).display().to_string();
        return Ok(Response::redirect(redirect));
    }

    let canonical = content_path.canonical();
    CAREERS.render(&content_path.into(), &canonical, cluster).await
}

#[get("/careers/apply/<title>", rank = 4)]
pub async fn careers_apply(title: PathBuf, cluster: &Cluster) -> Result<ResponseOk, crate::responses::NotFound> {
    let layout =
        crate::components::layouts::marketing::Base::new("Apply for a career", Some(&cluster)).no_transparent_nav();

    let job_title = title.display().to_string().replace("-", " ");
    let page = crate::components::pages::careers::Apply::new().job_title(&job_title);

    Ok(ResponseOk(layout.render(page)))
}

/// Redirect api to open-source
#[get("/docs/api/<path..>")]
pub async fn api_redirect(path: PathBuf) -> Redirect {
    match path.to_str().unwrap() {
        "apis" => Redirect::permanent("/docs/open-source/overview"),
        "client-sdk/search" => {
            Redirect::permanent("/docs/open-source/pgml/guides/improve-search-results-with-machine-learning")
        }
        "client-sdk/getting-started" => Redirect::permanent("/docs/open-source/pgml/guides/getting-started"),
        "sql-extensions/pgml.predict/" => Redirect::permanent("/docs/open-source/pgml/api/pgml.predict/"),
        "sql-extensions/pgml.deploy" => Redirect::permanent("/docs/open-source/pgml/api/pgml.deploy"),
        _ => Redirect::permanent("/docs/open-source/".to_owned() + path.to_str().unwrap()),
    }
}

/// Redirect our old sql-extension path.
#[get("/docs/open-source/sql-extension/<path..>")]
pub async fn sql_extension_redirect(path: PathBuf) -> Redirect {
    Redirect::permanent("/docs/open-source/pgml/api/".to_owned() + path.to_str().unwrap())
}

/// Redirect our old pgcat path.
#[get("/docs/product/pgcat/<path..>")]
pub async fn pgcat_redirect(path: PathBuf) -> Redirect {
    Redirect::permanent("/docs/open-source/pgcat/".to_owned() + path.to_str().unwrap())
}

/// Redirect our old cloud-database path.
#[get("/docs/product/cloud-database/<path..>")]
pub async fn cloud_database_redirect(path: PathBuf) -> Redirect {
    let path = path.to_str().unwrap();
    if path.is_empty() {
        Redirect::permanent("/docs/cloud/overview")
    } else {
        Redirect::permanent("/docs/cloud/".to_owned() + path)
    }
}

/// Redirect our old pgml docs.
#[get("/docs/open-source/client-sdk/<path..>")]
pub async fn pgml_redirect(path: PathBuf) -> Redirect {
    Redirect::permanent("/docs/open-source/korvus/api/".to_owned() + path.to_str().unwrap())
}

#[get("/docs/<path..>", rank = 5)]
async fn get_docs(
    path: PathBuf,
    cluster: &Cluster,
    origin: &Origin<'_>,
) -> Result<Response, crate::responses::NotFound> {
    use crate::components::{layouts::Docs, pages::docs::Article};

    let content_path = DOCS.get_content_path(path, origin).await;

    if content_path.redirect() {
        let redirect = Path::new("/docs/").join(content_path.path()).display().to_string();
        return Ok(Response::redirect(redirect));
    }

    if let Ok(doc) = Document::from_path(&content_path.clone().into()).await {
        if !doc.ignore() {
            let index = DOCS.open_index(&doc.path);

            let layout = Docs::new(&doc.title, Some(cluster))
                .index(&index)
                .image(&doc.thumbnail)
                .canonical(&content_path.canonical());

            let page = Article::new(&cluster).toc_links(&doc.toc_links).content(&doc.html());

            return Ok(Response::ok(layout.render(page)));
        }
    }

    let layout = crate::components::layouts::Docs::new("404", Some(cluster)).index(&DOCS.index);
    let page = crate::components::pages::docs::Article::new(&cluster).document_not_found();

    Err(crate::responses::NotFound(layout.render(page)))
}

#[get("/blog")]
async fn blog_landing_page(cluster: &Cluster) -> Result<ResponseOk, crate::responses::NotFound> {
    let layout = Base::new(
        "PostgresML blog landing page, home of technical tutorials, general updates and all things AI/ML.",
        Some(cluster),
    )
    .theme(Theme::Docs)
    .footer(cluster.context.marketing_footer.to_string());

    let mut index = Vec::new();

    let urls = BLOG.get_all_urls();

    for url in urls {
        let doc = Document::from_url(&url).await.unwrap();
        let meta = article_preview::DocMeta::from_document(doc);
        index.push(meta)
    }

    let featured_cards = index
        .clone()
        .into_iter()
        .filter(|x| x.featured)
        .collect::<Vec<article_preview::DocMeta>>();

    Ok(ResponseOk(layout.render(
        crate::components::pages::blog::LandingPage::new(cluster).featured_cards(featured_cards),
    )))
}

#[get("/docs")]
async fn docs_landing_page(cluster: &Cluster) -> Result<ResponseOk, crate::responses::NotFound> {
    let index = DOCS.open_index(&PathBuf::from("/docs"));

    let doc_layout = crate::components::layouts::Docs::new("Documentation", Some(cluster)).index(&index);

    let page = crate::components::pages::docs::LandingPage::new(&cluster)
        .parse_sections(DOCS.index.clone())
        .await;

    Ok(ResponseOk(doc_layout.render(page)))
}

/// Redirect our old MkDocs paths to the new ones under `/docs`.
#[get("/user_guides/<path..>", rank = 5)]
async fn get_user_guides(path: PathBuf) -> Result<Response, crate::responses::NotFound> {
    Ok(Response::redirect(format!("/docs/{}", path.display().to_string())))
}

#[get("/careers")]
async fn careers_landing_page(cluster: &Cluster) -> Result<ResponseOk, crate::responses::NotFound> {
    let layout = Base::new(
        "PostgresML careers landing page, Join us to help build the future of AI infrastructure.",
        Some(cluster),
    )
    .theme(Theme::Marketing);

    let page = crate::components::pages::careers::LandingPage::new(cluster)
        .index(&CAREERS)
        .await;

    Ok(ResponseOk(layout.render(page)))
}

#[get("/components-library-demo?<search>")]
async fn demo(search: Option<String>) -> Result<Response, Error> {
    #[cfg(not(debug_assertions))]
    {
        let _search = search;
        return Ok(Response::not_found());
    }

    #[cfg(debug_assertions)]
    {
        use crate::components::dropdown::{DropdownFrame, DropdownItems};
        use crate::components::inputs::text::search::SearchOption;
        if let Some(search) = search {
            let candidates = vec!["hello", "world", "foo", "bar"]
                .into_iter()
                .filter(|c| c.starts_with(&search))
                .map(|c| SearchOption::new(c.into()).into())
                .collect::<Vec<pgml_components::Component>>();

            Ok(Response::ok(
                DropdownFrame::rendered("model-search", DropdownItems::new(candidates).into()).render_once()?,
            ))
        } else {
            let layout = Base::new("Demos", None).theme(Theme::Marketing);

            let page = crate::components::pages::demo::Demo::new();
            Ok(Response::ok(layout.render(page)))
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![
        blog_landing_page,
        docs_landing_page,
        careers_landing_page,
        careers_apply,
        get_blog,
        get_blog_asset,
        get_careers,
        get_careers_asset,
        get_docs,
        get_docs_asset,
        get_user_guides,
        search,
        search_blog,
        demo,
        sql_extension_redirect,
        api_redirect,
        pgcat_redirect,
        pgml_redirect,
        cloud_database_redirect
    ]
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::markdown::options;
    use regex::Regex;
    use rocket::http::Status;
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
            // .manage(crate::utils::markdown::SiteSearch::new().await.expect("Error initializing site search"))
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

    // Ensure Docs render and there are no unparsed gitbook compnents.
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
            // Don't parse landing page since it is not markdown.
            if url != "/docs" {
                let path = collection.url_to_path(url.as_ref());
                crate::api::cms::Document::from_path(&path).await.unwrap();
            }
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
