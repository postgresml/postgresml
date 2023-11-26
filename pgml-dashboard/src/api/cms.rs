use std::path::{Path, PathBuf};

use comrak::{format_html_with_plugins, parse_document, Arena, ComrakPlugins};
use lazy_static::lazy_static;
use markdown::mdast::Node;
use rocket::{
    fs::NamedFile,
    http::{uri::Origin, Status},
    route::Route,
    State,
};
use yaml_rust::YamlLoader;

use crate::{
    components::cms::index_link::IndexLink,
    guards::Cluster,
    responses::{ResponseOk, Template},
    templates::docs::*,
    utils::config,
};

lazy_static! {
    static ref BLOG: Collection = Collection::new("Blog", true);
    static ref CAREERS: Collection = Collection::new("Careers", true);
    static ref DOCS: Collection = Collection::new("Docs", false);
}

/// A Gitbook collection of documents
#[derive(Default)]
struct Collection {
    /// The properly capitalized identifier for this collection
    name: String,
    /// The root location on disk for this collection
    root_dir: PathBuf,
    /// The root location for gitbook assets
    asset_dir: PathBuf,
    /// The base url for this collection
    url_root: PathBuf,
    /// A hierarchical list of content in this collection
    index: Vec<IndexLink>,
}

impl Collection {
    pub fn new(name: &str, hide_root: bool) -> Collection {
        info!("Loading collection: {name}");
        let name = name.to_owned();
        let slug = name.to_lowercase();
        let root_dir = config::cms_dir().join(&slug);
        let asset_dir = root_dir.join(".gitbook").join("assets");
        let url_root = PathBuf::from("/").join(&slug);

        let mut collection = Collection {
            name,
            root_dir,
            asset_dir,
            url_root,
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
    ) -> Result<ResponseOk, Status> {
        info!("get_content: {} | {path:?}", self.name);

        if origin.path().ends_with("/") {
            path = path.join("README");
        }

        let path = self.root_dir.join(path.with_extension("md"));

        self.render(&path, cluster, self).await
    }

    /// Create an index of the Collection based on the SUMMARY.md from Gitbook.
    /// Summary provides document ordering rather than raw filesystem access,
    /// in addition to formatted titles and paths.
    fn build_index(&mut self, hide_root: bool) {
        let summary_path = self.root_dir.join("SUMMARY.md");
        let summary_contents = std::fs::read_to_string(&summary_path)
            .expect(format!("Could not read summary: {summary_path:?}").as_str());
        let mdast = markdown::to_mdast(&summary_contents, &::markdown::ParseOptions::default())
            .expect(format!("Could not parse summary: {summary_path:?}").as_str());

        for node in mdast
            .children()
            .expect(format!("Summary has no content: {summary_path:?}").as_str())
            .iter()
        {
            match node {
                Node::List(list) => {
                    self.index = self.get_sub_links(&list).expect(
                        format!("Could not parse list of index links: {summary_path:?}").as_str(),
                    );
                    break;
                }
                _ => {
                    warn!("Irrelevant content ignored in: {summary_path:?}")
                }
            }
        }

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
                                                        let parent =
                                                            IndexLink::new(text.value.as_str())
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

    async fn render<'a>(
        &self,
        path: &'a PathBuf,
        cluster: &Cluster,
        collection: &Collection,
    ) -> Result<ResponseOk, Status> {
        // Read to string0
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
        let (description, contents) = if parts.len() > 1 {
            match YamlLoader::load_from_str(parts[1]) {
                Ok(meta) => {
                    if !meta.is_empty() {
                        let meta = meta[0].clone();
                        if meta.as_hash().is_none() {
                            (None, contents.to_string())
                        } else {
                            let description: Option<String> = match meta["description"]
                                .is_badvalue()
                            {
                                true => None,
                                false => Some(meta["description"].as_str().unwrap().to_string()),
                            };

                            (description, parts[2..].join("---").to_string())
                        }
                    } else {
                        (None, contents.to_string())
                    }
                }
                Err(_) => (None, contents.to_string()),
            }
        } else {
            (None, contents.to_string())
        };

        // Parse Markdown
        let arena = Arena::new();
        let root = parse_document(&arena, &contents, &crate::utils::markdown::options());

        // Title of the document is the first (and typically only) <h1>
        let title = crate::utils::markdown::get_title(&root).unwrap();
        let toc_links = crate::utils::markdown::get_toc(&root).unwrap();
        let image = crate::utils::markdown::get_image(&root);
        crate::utils::markdown::wrap_tables(&root, &arena).unwrap();

        // MkDocs syntax support, e.g. tabs, notes, alerts, etc.
        crate::utils::markdown::mkdocs(&root, &arena).unwrap();

        // Style headings like we like them
        let mut plugins = ComrakPlugins::default();
        let headings = crate::utils::markdown::MarkdownHeadings::new();
        plugins.render.heading_adapter = Some(&headings);
        plugins.render.codefence_syntax_highlighter =
            Some(&crate::utils::markdown::SyntaxHighlighter {});

        // Render
        let mut html = vec![];
        format_html_with_plugins(
            root,
            &crate::utils::markdown::options(),
            &mut html,
            &plugins,
        )
        .unwrap();
        let html = String::from_utf8(html).unwrap();

        // Handle navigation
        // TODO organize this functionality in the collection to cleanup
        let index: Vec<IndexLink> = self
            .index
            .clone()
            .iter_mut()
            .map(|nav_link| {
                let mut nav_link = nav_link.clone();
                nav_link.should_open(&path);
                nav_link
            })
            .collect();

        let user = if cluster.context.user.is_anonymous() {
            None
        } else {
            Some(cluster.context.user.clone())
        };

        let mut layout = crate::templates::Layout::new(&title);
        if let Some(image) = image {
            // translate relative url into absolute for head social sharing
            let parts = image.split(".gitbook/assets/").collect::<Vec<&str>>();
            let image_path = collection.url_root.join(".gitbook/assets").join(parts[1]);
            layout.image(config::asset_url(image_path.to_string_lossy()).as_ref());
        }
        if description.is_some() {
            layout.description(&description.unwrap());
        }
        if user.is_some() {
            layout.user(&user.unwrap());
        }

        let layout = layout
            .nav_title(&self.name)
            .nav_links(&index)
            .toc_links(&toc_links)
            .footer(cluster.context.marketing_footer.to_string());

        Ok(ResponseOk(
            layout.render(crate::templates::Article { content: html }),
        ))
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
) -> Result<ResponseOk, Status> {
    BLOG.get_content(path, cluster, origin).await
}

#[get("/careers/<path..>", rank = 5)]
async fn get_careers(
    path: PathBuf,
    cluster: &Cluster,
    origin: &Origin<'_>,
) -> Result<ResponseOk, Status> {
    CAREERS.get_content(path, cluster, origin).await
}

#[get("/docs/<path..>", rank = 5)]
async fn get_docs(
    path: PathBuf,
    cluster: &Cluster,
    origin: &Origin<'_>,
) -> Result<ResponseOk, Status> {
    DOCS.get_content(path, cluster, origin).await
}

pub fn routes() -> Vec<Route> {
    routes![
        get_blog,
        get_blog_asset,
        get_careers,
        get_careers_asset,
        get_docs,
        get_docs_asset,
        search
    ]
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

        crate::utils::markdown::wrap_tables(&root, &arena).unwrap();

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

        crate::utils::markdown::wrap_tables(&root, &arena).unwrap();

        let mut html = vec![];
        format_html_with_plugins(root, &options(), &mut html, &plugins).unwrap();
        let html = String::from_utf8(html).unwrap();

        assert!(
            !html.contains(r#"<div class="overflow-auto w-100">"#) || !html.contains(r#"</div>"#)
        );
    }
}
