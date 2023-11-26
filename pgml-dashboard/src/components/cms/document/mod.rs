use std::path::PathBuf;
use crate::templates::docs::TocLink;

pub struct Document {
    title: String,
    description: String,
    image: PathBuf,
    mdast: String,
    html: String,
    children: Vec<Document>,
    toc_links: Vec<TocLink>
}

impl Document {

    pub async fn render_html(&mut self) -> String {
        let document = None;
        for path in path.components() {
            for content in self.index() {

            }
        }

        if origin.path().ends_with("/") {
            path = path.join("README");
        }

        let path = self.root_dir.join(path.with_extension("md"));

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
        let ((image, description), contents) = if parts.len() > 1 {
            match YamlLoader::load_from_str(parts[1]) {
                Ok(meta) => {
                    if !meta.is_empty() {
                        let meta = meta[0].clone();
                        if meta.as_hash().is_none() {
                            ((None, None), contents.to_string())
                        } else {
                            let description: Option<String> = match meta["description"]
                                .is_badvalue()
                            {
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
        let root = parse_document(&arena, &contents, &crate::utils::markdown::options());

        // Title of the document is the first (and typically only) <h1>
        let title = crate::utils::markdown::get_title(&root).unwrap();
        let toc_links = crate::utils::markdown::get_toc(&root).unwrap();

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
        let index = self.index.clone();
        // into_iter().map(|nav_link| {
        // nav_link.should_open(&url, Path::new(&self.name.to_lowercase()))
        //}).collect();
    }
}

component!(Document);
