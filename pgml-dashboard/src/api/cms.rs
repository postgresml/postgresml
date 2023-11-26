use std::path::PathBuf;

use comrak::{format_html_with_plugins, parse_document, Arena, ComrakPlugins};
use lazy_static::lazy_static;
use rocket::{
    fs::NamedFile,
    http::{uri::Origin, Status},
    route::Route,
    State,
};
use tantivy::HasLen;

use crate::{
    components::cms::{Collection, Document},
    guards::Cluster,
    responses::{ResponseOk, Template},
    templates::docs::*,
    utils::config
};

lazy_static! {
    static ref BLOG: Collection = Collection::new("Blog");
    static ref CAREERS: Collection = Collection::new("Careers");
    static ref DOCS: Collection = Collection::new("Docs");
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
pub async fn get_blog_asset(path: PathBuf) -> Option<NamedFile> {
    BLOG.get_asset(&path).await
}

#[get("/careers/.gitbook/assets/<path>", rank = 10)]
pub async fn get_careers_asset(path: PathBuf) -> Option<NamedFile> {
    CAREERS.get_asset(&path).await
}

#[get("/docs/.gitbook/assets/<path>", rank = 10)]
pub async fn get_docs_asset(path: PathBuf) -> Option<NamedFile> {
    DOCS.get_asset(&path).await
}

#[get("/blog/<path..>", rank = 5)]
async fn get_blog(
    path: PathBuf,
    cluster: &Cluster
) -> Result<ResponseOk, Status> {
    get_document(&BLOG, &path, cluster).await
}

#[get("/careers/<path..>", rank = 5)]
async fn get_careers(
    path: PathBuf,
    cluster: &Cluster,
) -> Result<ResponseOk, Status> {
    get_document(&CAREERS, &path, cluster).await
}

#[get("/docs/<path..>", rank = 5)]
async fn get_docs(
    path: PathBuf,
    cluster: &Cluster,
) -> Result<ResponseOk, Status> {
    get_document(&DOCS, &path, cluster).await
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
