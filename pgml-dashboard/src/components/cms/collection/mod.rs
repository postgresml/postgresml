use std::path::{Path, PathBuf};

use markdown::mdast::Node;
use rocket::{
    fs::NamedFile
};
use yaml_rust::YamlLoader;

use crate::{
    components::cms::Document,
    responses::Response,
    guards::Cluster,
    responses::{ResponseOk, Template},
    templates::docs::*,
    utils::config
};

/// A Gitbook collection of documents
#[derive(Default)]
pub struct Collection {
    /// The properly capitalized identifier for this collection
    name: String,
    /// The url & path friendly identifier for this collection
    slug: String,
    /// The root location on disk for this collection
    root_dir: PathBuf,
    /// The root location for gitbook assets
    asset_dir: PathBuf,
    /// The base url for this collection
    url_root: PathBuf,
    /// A hierarchical list of content in this collection
    doc_root: Document,
}

impl Collection {
    pub fn new(name: &str) -> Collection {
        info!("Loading collection: {name}");
        let name = name.to_owned();
        let slug = name.to_lowercase();
        let root_dir = config::cms_dir().join(&slug);
        let asset_dir = root_dir.join(".gitbook").join("assets");
        let url_root = PathBuf::from("/").join(&slug);

        let mut collection = Collection {
            name,
            slug,
            root_dir,
            asset_dir,
            url_root,
            ..Default::default()
        };
        collection.build_index();
        collection
    }

    pub async fn get_asset(&self, path: &PathBuf) -> Option<NamedFile> {
        info!("get_asset: {} {path}", self.name);
        NamedFile::open(self.asset_dir.join(path)).await.ok()
    }

    pub async fn get_content(
        &self,
        path: &PathBuf,
    ) -> Response {
        info!("get_content: {} | {path:?}", self.name);
        let document = match self.get_document(&path) {
            Some(document) => document,
            None => {
                let mut layout = crate::templates::Layout::new("Not Found");
                let html = layout.render(crate::templates::Article { content: None });
                return Response::not_found(html)
            }
        };

        let mut layout = crate::templates::Layout::new(&document.title);
        if let Some(image) = document.image {
            layout.image(&image);
        }
        if let Some(description) = document.description {
            layout.description(&description);
        }
        if let Some(user) = cluster.context.user {
            layout.user(&user);
        }

        let layout = layout
            .nav_title(&document.title)
            .nav_links(&collection.index_links)
            .toc_links(&document.toc_links)
            .footer(cluster.context.marketing_footer.to_string());

        Response::ok(
            layout.render(crate::templates::Article { content: document.html })
        )
   }

    /// Create an index of the Collection based on the SUMMARY.md from Gitbook.
    /// Summary provides document ordering rather than raw filesystem access,
    /// in addition to formatted titles and paths.
    fn build_index(&mut self) {
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
                        format!("Could not parse list of index links: {summary_path:?}")
                            .as_str(),
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
}

component!(Collection);
