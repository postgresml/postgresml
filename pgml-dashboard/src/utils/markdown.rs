use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use comrak::{
    adapters::{HeadingAdapter, HeadingMeta, SyntaxHighlighterAdapter},
    arena_tree::Node,
    nodes::{Ast, AstNode, NodeValue},
    parse_document, Arena, ComrakExtensionOptions, ComrakOptions, ComrakRenderOptions,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use tantivy::collector::TopDocs;
use tantivy::query::{QueryParser, RegexQuery};
use tantivy::schema::*;
use tantivy::tokenizer::{LowerCaser, NgramTokenizer, TextAnalyzer};
use tantivy::{Index, IndexReader, SnippetGenerator};

use crate::{templates::docs::TocLink, utils::config};

pub struct MarkdownHeadings {}
impl HeadingAdapter for MarkdownHeadings {
    fn enter(&self, meta: &HeadingMeta) -> String {
        let mut s = DefaultHasher::new();

        meta.content
            .to_string()
            .to_lowercase()
            .replace(" ", "-")
            .hash(&mut s);
        let id = "header-".to_string() + &s.finish().to_string();

        match meta.level {
            1 => format!(r#"<h1 class="h1 mb-5" id="{id}">"#),
            2 => format!(r#"<h2 class="h2 mb-4 mt-5" id="{id}">"#),
            3 => format!(r#"<h3 class="h3 mb-4 mt-5" id="{id}">"#),
            4 => format!(r#"<h4 class="h5 mb-3 mt-3" id="{id}">"#),
            5 => format!(r#"<h5 class="h6 mb-2 mt-4" id="{id}">"#),
            6 => format!(r#"<h6 class="h6 mb-1 mt-1" id="{id}">"#),
            _ => unreachable!(),
        }
    }

    fn exit(&self, meta: &HeadingMeta) -> String {
        match meta.level {
            1 => r#"</h1>"#,
            2 => r#"</h2>"#,
            3 => r#"</h3>"#,
            4 => r#"</h4>"#,
            5 => r#"</h5>"#,
            6 => r#"</h6>"#,
            _ => unreachable!(),
        }
        .into()
    }
}

pub struct SyntaxHighlighter {}
impl SyntaxHighlighterAdapter for SyntaxHighlighter {
    fn highlight(&self, lang: Option<&str>, code: &str) -> String {
        let code = if lang.is_some() {
            let code = code.to_string();
            let lang = lang.unwrap();

            match lang {
                "postgresql" | "sql" | "postgresql-line-nums" => {
                    lazy_static! {
                        static ref SQL_KEYS: [&'static str; 57] = [
                            "CASCADE",
                            "INNER ",
                            "ON ",
                            "WITH",
                            "SELECT",
                            "UPDATE",
                            "DELETE",
                            "WHERE",
                            "AS",
                            "HAVING",
                            "ORDER BY",
                            "ASC",
                            "DESC",
                            "LIMIT",
                            "FROM",
                            "CREATE",
                            "REPLACE",
                            "DROP",
                            "VIEW",
                            "EXTENSION",
                            "SERVER",
                            "FOREIGN DATA WRAPPER",
                            "OPTIONS",
                            "IMPORT FOREIGN SCHEMA",
                            "CREATE USER MAPPING",
                            "INTO",
                            "PUBLICATION",
                            "FOR",
                            "ALL",
                            "TABLES",
                            "CONNECTION",
                            "SUBSCRIPTION",
                            "JOIN",
                            "INTO",
                            "INSERT",
                            "BEGIN",
                            "ALTER",
                            "SCHEMA",
                            "RENAME",
                            "COMMIT",
                            "AND ",
                            "ADD COLUMN",
                            "ALTER TABLE",
                            "PRIMARY KEY",
                            "DO",
                            "END",
                            "BETWEEN",
                            "SET",
                            "INDEX",
                            "USING",
                            "GROUP BY",
                            "CREATE TABLE",
                            "pgml.embed",
                            "pgml.sum",
                            "pgml.norm_l2",
                            "CONCURRENTLY",
                            "ON",
                        ];
                        static ref SQL_KEYS_REPLACEMENTS: [&'static str; 57] = [
                            "<span class=\"syntax-highlight\">CASCADE</span>",
                            "<span class=\"syntax-highlight\">INNER </span>",
                            "<span class=\"syntax-highlight\">ON </span>",
                            "<span class=\"syntax-highlight\">WITH</span>",
                            "<span class=\"syntax-highlight\">SELECT</span>",
                            "<span class=\"syntax-highlight\">UPDATE</span>",
                            "<span class=\"syntax-highlight\">DELETE</span>",
                            "<span class=\"syntax-highlight\">WHERE</span>",
                            "<span class=\"syntax-highlight\">AS</span>",
                            "<span class=\"syntax-highlight\">HAVING</span>",
                            "<span class=\"syntax-highlight\">ORDER BY</span>",
                            "<span class=\"syntax-highlight\">ASC</span>",
                            "<span class=\"syntax-highlight\">DESC</span>",
                            "<span class=\"syntax-highlight\">LIMIT</span>",
                            "<span class=\"syntax-highlight\">FROM</span>",
                            "<span class=\"syntax-highlight\">CREATE</span>",
                            "<span class=\"syntax-highlight\">REPLACE</span>",
                            "<span class=\"syntax-highlight\">DROP</span>",
                            "<span class=\"syntax-highlight\">VIEW</span>",
                            "<span class=\"syntax-highlight\">EXTENSION</span>",
                            "<span class=\"syntax-highlight\">SERVER</span>",
                            "<span class=\"syntax-highlight\">FOREIGN DATA WRAPPER</span>",
                            "<span class=\"syntax-highlight\">OPTIONS</span>",
                            "<span class=\"syntax-highlight\">IMPORT FOREIGN SCHEMA</span>",
                            "<span class=\"syntax-highlight\">CREATE USER MAPPING</span>",
                            "<span class=\"syntax-highlight\">INTO</span>",
                            "<span class=\"syntax-highlight\">PUBLICATION</span>",
                            "<span class=\"syntax-highlight\">FOR</span>",
                            "<span class=\"syntax-highlight\">ALL</span>",
                            "<span class=\"syntax-highlight\">TABLES</span>",
                            "<span class=\"syntax-highlight\">CONNECTION</span>",
                            "<span class=\"syntax-highlight\">SUBSCRIPTION</span>",
                            "<span class=\"syntax-highlight\">JOIN</span>",
                            "<span class=\"syntax-highlight\">INTO</span>",
                            "<span class=\"syntax-highlight\">INSERT</span>",
                            "<span class=\"syntax-highlight\">BEGIN</span>",
                            "<span class=\"syntax-highlight\">ALTER</span>",
                            "<span class=\"syntax-highlight\">SCHEMA</span>",
                            "<span class=\"syntax-highlight\">RENAME</span>",
                            "<span class=\"syntax-highlight\">COMMIT</span>",
                            "<span class=\"syntax-highlight\">AND </span>",
                            "<span class=\"syntax-highlight\">ADD COLUMN</span>",
                            "<span class=\"syntax-highlight\">ALTER TABLE</span>",
                            "<span class=\"syntax-highlight\">PRIMARY KEY</span>",
                            "<span class=\"syntax-highlight\">DO</span>",
                            "<span class=\"syntax-highlight\">END</span>",
                            "<span class=\"syntax-highlight\">BETWEEN</span>",
                            "<span class=\"syntax-highlight\">SET</span>",
                            "<span class=\"syntax-highlight\">INDEX</span>",
                            "<span class=\"syntax-highlight\">USING</span>",
                            "<span class=\"syntax-highlight\">GROUP BY</span>",
                            "<span class=\"syntax-highlight\">CREATE TABLE</span>",
                            "<strong>pgml.embed</strong>",
                            "<strong>pgml.sum</strong>",
                            "<strong>pgml.norm_l2</strong>",
                            "<span class=\"syntax-highlight\">CONCURRENTLY</span>",
                            "<span class=\"syntax-highlight\">ON</span>",
                        ];
                        static ref AHO_SQL: AhoCorasick = AhoCorasickBuilder::new()
                            .match_kind(MatchKind::LeftmostLongest)
                            .build(SQL_KEYS.iter());
                    }
                    let code = if lang == "postgresql-line-nums" {
                        let mut code = code.split("\n")
                            .into_iter()
                            .enumerate()
                            .map(|(index, code)| format!("<span style=\"user-select: none; {}\" class=\"text-muted\">{}.</span><span class=\"code-content\">{}</span>",  if index < 9 { "padding-right: .75rem" } else { "padding-right: .50rem" }, index+1, code))
                            .collect::<Vec<String>>();
                        code.pop();
                        code.into_iter().join("\n")
                    } else {
                        code.split("\n")
                            .map(|code| format!("<span class=\"code-content\">{}</span>", code))
                            .join("\n")
                    };

                    let code = AHO_SQL
                        .replace_all(&code, &SQL_KEYS_REPLACEMENTS[..])
                        .to_string();
                    code
                }

                "bash" => {
                    lazy_static! {
                        static ref RE_BASH: regex::Regex = regex::Regex::new(r"(cd)").unwrap();
                    }

                    RE_BASH
                        .replace_all(&code, r#"<span class="syntax-highlight">$1</span>"#)
                        .to_string()
                }

                "python" => {
                    lazy_static! {
                        static ref RE_PYTHON: regex::Regex =
                            regex::Regex::new(r"(import |def |return )").unwrap();
                    }

                    RE_PYTHON
                        .replace_all(&code, r#"<span class="syntax-highlight">$1</span>"#)
                        .to_string()
                }

                _ => code,
            }
        } else {
            code.to_string()
        };

        String::from(format!(
            "<span class=\"lang-{}\">{}</span>",
            lang.unwrap(),
            code
        ))
    }

    fn build_pre_tag(&self, _attributes: &HashMap<String, String>) -> String {
        String::from("<pre data-controller=\"copy\"><div class=\"code-toolbar\">
                <span data-action=\"click->copy#codeCopy\" class=\"material-symbols-outlined btn-code-toolbar\">content_copy</span>
                <span class=\"material-symbols-outlined btn-code-toolbar\" disabled>link</span>
                <span class=\"material-symbols-outlined btn-code-toolbar\" disabled>edit</span>
            </div>")
    }

    fn build_code_tag(&self, _attributes: &HashMap<String, String>) -> String {
        String::from("<code>")
    }
}

pub fn options() -> ComrakOptions {
    let mut options = ComrakOptions::default();

    let mut render_options = ComrakRenderOptions::default();
    render_options.unsafe_ = true;

    options.extension = ComrakExtensionOptions {
        strikethrough: true,
        tagfilter: false,
        table: true,
        autolink: true,
        tasklist: true,
        superscript: true,
        header_ids: Some("pgml-".to_string()),
        footnotes: true,
        description_lists: true,
        front_matter_delimiter: None,
    };
    options.render = render_options;

    options
}

/// Iterate through the document tree and call function F on all nodes.
fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &mut F) -> anyhow::Result<()>
where
    F: FnMut(&'a AstNode<'a>) -> anyhow::Result<bool>,
{
    let continue_ = f(node)?;

    if continue_ {
        for c in node.children() {
            iter_nodes(c, f)?;
        }
    }

    Ok(())
}

/// Get the title of the article.
///
/// # Arguments
///
/// * `root` - The root node of the document tree.
///
pub fn get_title<'a>(root: &'a AstNode<'a>) -> anyhow::Result<String> {
    let mut title = String::new();

    iter_nodes(root, &mut |node| {
        match &node.data.borrow().value {
            &NodeValue::Heading(ref header) => {
                if header.level == 1 {
                    let sibling = node
                        .first_child()
                        .ok_or(anyhow::anyhow!("markdown heading has no child"))?;
                    match &sibling.data.borrow().value {
                        &NodeValue::Text(ref text) => {
                            title = text.to_owned();
                            return Ok(false);
                        }
                        _ => (),
                    };
                }
            }
            _ => (),
        };

        Ok(true)
    })?;

    Ok(title)
}

/// Generate the table of contents for the article.
///
/// # Arguments
///
/// * `root` - The root node of the document tree.
///
pub fn get_toc<'a>(root: &'a AstNode<'a>) -> anyhow::Result<Vec<TocLink>> {
    let mut links = Vec::new();

    iter_nodes(root, &mut |node| {
        match &node.data.borrow().value {
            &NodeValue::Heading(ref header) => {
                if header.level != 1 {
                    let sibling = node
                        .first_child()
                        .ok_or(anyhow::anyhow!("markdown heading has no child"))?;
                    match &sibling.data.borrow().value {
                        &NodeValue::Text(ref text) => {
                            links.push(TocLink::new(text).level(header.level));
                            return Ok(false);
                        }
                        _ => (),
                    };
                }
            }
            _ => (),
        };

        Ok(true)
    })?;

    Ok(links)
}

/// Get all indexable text from the document.
///
/// # Arguments
///
/// * `root` - The root node of the document tree.
///
pub fn get_text<'a>(root: &'a AstNode<'a>) -> anyhow::Result<Vec<String>> {
    let mut texts = Vec::new();

    iter_nodes(root, &mut |node| match &node.data.borrow().value {
        &NodeValue::Text(ref text) => {
            // Skip markdown annotations
            if text.starts_with("!!!") || text.starts_with("===") {
                Ok(true)
            } else {
                texts.push(text.to_owned());
                Ok(true)
            }
        }

        &NodeValue::Table(_) => Ok(true),

        &NodeValue::Image(_) => Ok(false),

        &NodeValue::Code(ref node) => {
            texts.push(node.literal.to_owned());
            Ok(true)
        }

        &NodeValue::CodeBlock(ref _node) => {
            // Not a good idea to index code yet I think, gets too messy.
            // texts.push(String::from_utf8_lossy(&node.literal).to_string());
            Ok(false)
        }

        _ => Ok(true),
    })?;

    Ok(texts)
}

struct Tab<'a> {
    children: Vec<&'a AstNode<'a>>,
    name: String,
    id: String,
    active: bool,
}

impl<'a> Tab<'a> {
    fn new(name: String) -> Tab<'a> {
        Tab {
            children: vec![],
            name,
            id: crate::utils::secure::random_string(10),
            active: false,
        }
    }

    fn active(mut self) -> Tab<'a> {
        self.active = true;
        self
    }

    fn render(&self) -> String {
        let active = if self.active { "active" } else { "" };

        format!(
            "
            <li class=\"nav-item\" role=\"presentation\">
                <button
                    class=\"nav-link {active}\"
                    data-bs-toggle=\"tab\"
                    data-bs-target=\"#tab-{id}\"
                    type=\"button\"
                    role=\"tab\"
                    aria-controls=\"tab-{id}\"
                    aria-selected=\"true\">
                        {name}
                </button>
            </li>
        ",
            active = active,
            id = self.id,
            name = self.name
        )
    }
}

struct Admonition {
    class: String,
    icon: String,
    title: String,
}

impl Admonition {
    fn html(&self) -> String {
        format!(
            r#"
        <div class="{}">
            <div class="admonition-title">
                <div class="admonition-img">
                    <span class="material-symbols-outlined">{}</span>
                </div>
                {}
            </div>
        "#,
            self.class, self.icon, self.title
        )
    }
}

impl From<&str> for Admonition {
    fn from(utf8: &str) -> Admonition {
        let (class, icon, title) = if utf8.starts_with("!!! info") {
            ("admonition-info", "help", "Info")
        } else if utf8.starts_with("!!! note") {
            ("admonition-note", "priority_high", "Note")
        } else if utf8.starts_with("!!! abstract") {
            ("admonition-abstract", "sticky_note_2", "Abstract")
        } else if utf8.starts_with("!!! tip") {
            ("admonition-tip", "help", "Tip")
        } else if utf8.starts_with("!!! question") {
            ("admonition-question", "help", "Question")
        } else if utf8.starts_with("!!! example") {
            ("admonition-example", "code", "Example")
        } else if utf8.starts_with("!!! success") {
            ("admonition-success", "check_circle", "Success")
        } else if utf8.starts_with("!!! quote") {
            ("admonition-quote", "format_quote", "Quote")
        } else if utf8.starts_with("!!! bug") {
            ("admonition-bug", "bug_report", "Bug")
        } else if utf8.starts_with("!!! warning") {
            ("admonition-warning", "warning", "Warning")
        } else if utf8.starts_with("!!! fail") {
            ("admonition-fail", "dangerous", "Fail")
        } else if utf8.starts_with("!!! danger") {
            ("admonition-danger", "gpp_maybe", "Danger")
        } else {
            ("admonition-generic", "", "")
        };

        Self {
            class: String::from(class),
            icon: String::from(icon),
            title: String::from(title),
        }
    }
}

struct CodeBlock {
    time: Option<String>,
    title: Option<String>,
}

impl CodeBlock {
    fn html(&self, html_type: &str) -> Option<String> {
        match html_type {
            "time" => match &self.time {
                Some(time) => Some(format!(
                    r#"
                        <div class="execution-time">
                            <span class="material-symbols-outlined">timer</span>
                            {}
                        </div>
                    "#,
                    time
                )),
                None => None,
            },
            "code" => match &self.title {
                Some(title) => Some(format!(
                    r#"
                    <div class="code-block with-title">
                        <div class="title">
                            {}
                        </div>
                "#,
                    title
                )),
                None => Some(format!(
                    r#"
                    <div class="code-block">
                "#
                )),
            },
            "results" => match &self.title {
                Some(title) => Some(format!(
                    r#"
                        <div class="results with-title">
                            <div class="title">
                                {}
                            </div>
                    "#,
                    title
                )),
                None => Some(format!(
                    r#"
                    <div class="results">
                "#
                )),
            },
            _ => None,
        }
    }

    fn parser(utf8: &str, item: &str) -> Option<String> {
        let title_index = utf8.find(item);
        let (start, end) = match title_index {
            Some(index) => {
                let start = index + item.len();
                let title_length = utf8.to_string()[start..].find("\"");
                match title_length {
                    Some(title_length) => (start, start + title_length),
                    None => (0, 0),
                }
            }
            None => (0, 0),
        };

        if end - start > 0 {
            Some(format!("{}", &utf8[start..end]))
        } else {
            None
        }
    }
}

/// Convert MkDocs to Bootstrap.
///
/// Example:
///
/// === "SQL"
///
/// Something inside the tab (no ident because indent = code block)
///
/// === "Output"
///
/// Something inside the tab
///
/// ===
///
/// The last "===" closes the tab.
pub fn mkdocs<'a>(root: &'a AstNode<'a>, arena: &'a Arena<AstNode<'a>>) -> anyhow::Result<()> {
    let mut tabs = Vec::new();

    // tracks open !!! blocks and holds items to apppend prior to closing
    let mut info_block_close_items: Vec<Option<String>> = vec![];

    iter_nodes(root, &mut |node| {
        match &mut node.data.borrow_mut().value {
            &mut NodeValue::Text(ref mut text) => {
                if text.starts_with("=== \"") {
                    let mut parent = {
                        match node.parent() {
                            Some(parent) => parent,
                            None => node,
                        }
                    };

                    let tab = Tab::new(text.replace("=== ", "").replace("\"", ""));

                    if tabs.is_empty() {
                        let n =
                            arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                                r#"
                                <ul class="nav nav-tabs">
                            "#
                                .to_string()
                                .into(),
                            )))));

                        parent.insert_after(n);
                        parent.detach();
                        parent = n;
                    }

                    if tabs.is_empty() {
                        tabs.push(tab.active());
                    } else {
                        tabs.push(tab);
                    };

                    parent.insert_after(arena.alloc(Node::new(RefCell::new(Ast::new(
                        NodeValue::HtmlInline(tabs.last().unwrap().render().into()),
                    )))));

                    // Remove the "===" from the tree.
                    node.detach();
                } else if text.starts_with("===") {
                    let mut parent = {
                        match node.parent() {
                            Some(parent) => parent,
                            None => node,
                        }
                    };

                    if !tabs.is_empty() {
                        let n = arena.alloc(Node::new(RefCell::new(Ast::new(
                            NodeValue::HtmlInline("</ul>".to_string().into()),
                        ))));

                        parent.insert_after(n);
                        parent.detach();

                        parent = n;

                        let n =
                            arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                                r#"<div class="tab-content">"#.to_string().into(),
                            )))));

                        parent.insert_after(n);

                        parent = n;

                        for tab in tabs.iter() {
                            let r = arena.alloc(Node::new(RefCell::new(Ast::new(
                                NodeValue::HtmlInline(
                                    format!(
                                        r#"
                                                <div
                                                    class="tab-pane {active} pt-2"
                                                    id="tab-{id}"
                                                    role="tabpanel"
                                                    aria-labelledby="tab-{id}">
                                            "#,
                                        active = if tab.active { "show active" } else { "" },
                                        id = tab.id
                                    )
                                    .into(),
                                ),
                            ))));

                            for child in tab.children.iter() {
                                r.append(child);
                            }

                            parent.append(r);
                            parent = r;

                            let n = arena.alloc(Node::new(RefCell::new(Ast::new(
                                NodeValue::HtmlInline(r#"</div>"#.to_string().into()),
                            ))));

                            parent.insert_after(n);
                            parent = n;
                        }

                        parent.insert_after(arena.alloc(Node::new(RefCell::new(Ast::new(
                            NodeValue::HtmlInline(r#"</div>"#.to_string().into()),
                        )))));

                        tabs.clear();
                        node.detach();
                    }
                } else if text.starts_with("!!! info")
                    || text.starts_with("!!! bug")
                    || text.starts_with("!!! tip")
                    || text.starts_with("!!! note")
                    || text.starts_with("!!! abstract")
                    || text.starts_with("!!! example")
                    || text.starts_with("!!! warning")
                    || text.starts_with("!!! question")
                    || text.starts_with("!!! success")
                    || text.starts_with("!!! quote")
                    || text.starts_with("!!! fail")
                    || text.starts_with("!!! danger")
                    || text.starts_with("!!! generic")
                {
                    let parent = node.parent().unwrap();

                    let admonition: Admonition = Admonition::from(text.as_ref());

                    let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                        admonition.html().into(),
                    )))));

                    info_block_close_items.push(None);
                    parent.insert_after(n);
                    parent.detach();
                } else if text.starts_with("!!! code_block") {
                    let parent = node.parent().unwrap();

                    let title = CodeBlock::parser(text.as_ref(), r#"title=""#);
                    let time = CodeBlock::parser(text.as_ref(), r#"time=""#);
                    let code_block = CodeBlock { time, title };

                    match code_block.html("code") {
                        Some(html) => {
                            let n = arena.alloc(Node::new(RefCell::new(Ast::new(
                                NodeValue::HtmlInline(html.into()),
                            ))));
                            parent.insert_after(n);
                        }
                        None => (),
                    };

                    // add time ot info block to be appended prior to closing
                    info_block_close_items.push(code_block.html("time"));
                    parent.detach();
                } else if text.starts_with("!!! results") {
                    let parent = node.parent().unwrap();

                    let title = CodeBlock::parser(text.as_ref(), r#"title=""#);
                    let code_block = CodeBlock { time: None, title };

                    match code_block.html("results") {
                        Some(html) => {
                            let n = arena.alloc(Node::new(RefCell::new(Ast::new(
                                NodeValue::HtmlInline(html.into()),
                            ))));
                            parent.insert_after(n);
                        }
                        None => (),
                    }

                    info_block_close_items.push(None);
                    parent.detach();
                } else if text.starts_with("!!!") {
                    if info_block_close_items.len() > 0 {
                        let parent = node.parent().unwrap();

                        match info_block_close_items.pop() {
                            Some(html) => match html {
                                Some(html) => {
                                    let timing = arena.alloc(Node::new(RefCell::new(Ast::new(
                                        NodeValue::HtmlInline(format!("{html} </div>").into()),
                                    ))));
                                    parent.insert_after(timing);
                                }
                                None => {
                                    let n = arena.alloc(Node::new(RefCell::new(Ast::new(
                                        NodeValue::HtmlInline(
                                            r#"
                                        </div>
                                        "#
                                            .to_string()
                                            .into(),
                                        ),
                                    ))));

                                    parent.insert_after(n);
                                }
                            },
                            None => {
                                let n = arena.alloc(Node::new(RefCell::new(Ast::new(
                                    NodeValue::HtmlInline(
                                        r#"
                                </div>
                                "#
                                        .to_string()
                                        .into(),
                                    ),
                                ))));

                                parent.insert_after(n);
                            }
                        }

                        parent.detach();
                    }
                }

                // TODO montana
                // *text = text.as_bytes().to_vec();

                Ok(true)
            }

            _ => {
                if !tabs.is_empty() {
                    let last_tab = tabs.last_mut().unwrap();
                    let mut ancestors = node.ancestors();
                    let mut pushed = false;

                    // Check that we haven't pushed it's parent in yet.
                    while let Some(parent) = ancestors.next() {
                        pushed = last_tab
                            .children
                            .iter()
                            .filter(|node| node.same_node(parent))
                            .last()
                            .is_some();

                        if pushed {
                            break;
                        }
                    }

                    if !pushed {
                        last_tab.children.push(node);
                    }
                }

                Ok(true)
            }
        }
    })?;

    Ok(())
}

pub async fn get_document(path: &PathBuf) -> anyhow::Result<String> {
    Ok(tokio::fs::read_to_string(path).await?)
}

pub struct SearchResult {
    pub title: String,
    pub body: String,
    pub path: String,
    pub snippet: String,
}

pub struct SearchIndex {
    // The index.
    pub index: Arc<Index>,

    // Index schema (fields).
    pub schema: Arc<Schema>,

    // The index reader, supports concurrent access.
    pub reader: Arc<IndexReader>,
}

impl SearchIndex {
    pub fn path() -> PathBuf {
        Path::new(&config::search_index_dir()).to_owned()
    }

    pub fn documents() -> Vec<PathBuf> {
        let guides =
            glob::glob(&(config::content_dir() + "/docs/guides/**/*.md")).expect("glob failed");
        let blogs = glob::glob(&(config::content_dir() + "/blog/**/*.md")).expect("glob failed");
        guides
            .chain(blogs)
            .map(|path| path.expect("glob path failed"))
            .collect()
    }

    pub fn schema() -> Schema {
        // TODO: Make trigram title index
        // and full text body index, and use trigram only if body gets nothing.
        let mut schema_builder = Schema::builder();
        let title_field_indexing = TextFieldIndexing::default()
            .set_tokenizer("ngram3")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);
        let title_options = TextOptions::default()
            .set_indexing_options(title_field_indexing)
            .set_stored();

        schema_builder.add_text_field("title", title_options.clone());
        schema_builder.add_text_field("title_regex", TEXT | STORED);
        schema_builder.add_text_field("body", TEXT | STORED);
        schema_builder.add_text_field("path", STORED);

        schema_builder.build()
    }

    pub async fn build() -> tantivy::Result<()> {
        // Remove existing index.
        let _ = std::fs::remove_dir_all(Self::path());
        std::fs::create_dir(Self::path()).unwrap();

        let index = tokio::task::spawn_blocking(move || -> tantivy::Result<Index> {
            Ok(Index::create_in_dir(&Self::path(), Self::schema())?)
        })
        .await
        .unwrap()?;

        let ngram = TextAnalyzer::from(NgramTokenizer::new(3, 3, false)).filter(LowerCaser);

        index.tokenizers().register("ngram3", ngram);

        let schema = Self::schema();
        let mut index_writer = index.writer(50_000_000)?;

        for path in Self::documents().into_iter() {
            let text = get_document(&path).await.unwrap();

            let arena = Arena::new();
            let root = parse_document(&arena, &text, &options());
            let title_text = get_title(&root).unwrap();
            let body_text = get_text(&root).unwrap().into_iter().join(" ");

            let title_field = schema.get_field("title").unwrap();
            let body_field = schema.get_field("body").unwrap();
            let path_field = schema.get_field("path").unwrap();
            let title_regex_field = schema.get_field("title_regex").unwrap();

            let path = path
                .to_str()
                .unwrap()
                .to_string()
                .split("content")
                .last()
                .unwrap()
                .to_string();
            let mut doc = Document::default();
            doc.add_text(title_field, &title_text);
            doc.add_text(body_field, &body_text);
            doc.add_text(path_field, &path);
            doc.add_text(title_regex_field, &title_text);

            index_writer.add_document(doc)?;
        }

        tokio::task::spawn_blocking(move || -> tantivy::Result<u64> { Ok(index_writer.commit()?) })
            .await
            .unwrap()?;

        Ok(())
    }

    pub fn open() -> tantivy::Result<SearchIndex> {
        let index = tantivy::Index::open_in_dir(&Self::path())?;

        let reader = index.reader_builder().try_into()?;

        let ngram = TextAnalyzer::from(NgramTokenizer::new(3, 3, false)).filter(LowerCaser);

        index.tokenizers().register("ngram3", ngram);

        Ok(SearchIndex {
            index: Arc::new(index),
            schema: Arc::new(Self::schema()),
            reader: Arc::new(reader),
        })
    }

    pub fn search(&self, query_string: &str) -> tantivy::Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let searcher = self.reader.searcher();
        let title_field = self.schema.get_field("title").unwrap();
        let body_field = self.schema.get_field("body").unwrap();
        let path_field = self.schema.get_field("path").unwrap();
        let title_regex_field = self.schema.get_field("title_regex").unwrap();

        // Search using:
        //
        // 1. Full text search on the body
        // 2. Trigrams on the title
        let query_parser = QueryParser::for_index(&self.index, vec![title_field, body_field]);
        let query = match query_parser.parse_query(query_string) {
            Ok(query) => query,
            Err(err) => {
                warn!("Query parse error: {}", err);
                return Ok(Vec::new());
            }
        };

        let mut top_docs = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();

        // If that's not enough, search using prefix search on the title.
        if top_docs.len() < 10 {
            let query =
                match RegexQuery::from_pattern(&format!("{}.*", query_string), title_regex_field) {
                    Ok(query) => query,
                    Err(err) => {
                        warn!("Query regex error: {}", err);
                        return Ok(Vec::new());
                    }
                };

            let more_results = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
            top_docs.extend(more_results);
        }

        // Oh jeez ok
        if top_docs.len() < 10 {
            let query = match RegexQuery::from_pattern(&format!("{}.*", query_string), body_field) {
                Ok(query) => query,
                Err(err) => {
                    warn!("Query regex error: {}", err);
                    return Ok(Vec::new());
                }
            };

            let more_results = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
            top_docs.extend(more_results);
        }

        // Generate snippets for the FTS query.
        let snippet_generator = SnippetGenerator::create(&searcher, &*query, body_field)?;

        let mut dedup = HashSet::new();

        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            let snippet = snippet_generator.snippet_from_doc(&retrieved_doc);
            let path = retrieved_doc
                .get_first(path_field)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string()
                .replace(".md", "")
                .replace(&config::static_dir(), "");

            // Dedup results from prefix search and full text search.
            let new = dedup.insert(path.clone());

            if !new {
                continue;
            }

            let title = retrieved_doc
                .get_first(title_field)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string();
            let body = retrieved_doc
                .get_first(body_field)
                .unwrap()
                .as_text()
                .unwrap()
                .to_string();

            let snippet = if snippet.is_empty() {
                body.split(" ").take(20).collect::<Vec<&str>>().join(" ") + "&nbsp;..."
            } else {
                "...&nbsp;".to_string() + &snippet.to_html() + "&nbsp;..."
            };

            results.push(SearchResult {
                title,
                body,
                path,
                snippet,
            });
        }

        Ok(results)
    }
}
