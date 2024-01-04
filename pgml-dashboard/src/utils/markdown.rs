use crate::{templates::docs::TocLink, utils::config};

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use anyhow::Result;
use comrak::{
    adapters::{HeadingAdapter, HeadingMeta, SyntaxHighlighterAdapter},
    arena_tree::Node,
    nodes::{Ast, AstNode, NodeValue},
    parse_document, Arena, ComrakExtensionOptions, ComrakOptions, ComrakRenderOptions,
};
use itertools::Itertools;
use regex::Regex;
use tantivy::collector::TopDocs;
use tantivy::query::{QueryParser, RegexQuery};
use tantivy::schema::*;
use tantivy::tokenizer::{LowerCaser, NgramTokenizer, TextAnalyzer};
use tantivy::{Index, IndexReader, SnippetGenerator};
use url::Url;

use std::fmt;

pub struct MarkdownHeadings {
    counter: Arc<AtomicUsize>,
}

impl Default for MarkdownHeadings {
    fn default() -> Self {
        Self {
            counter: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl MarkdownHeadings {
    pub fn new() -> Self {
        Self::default()
    }
}

impl HeadingAdapter for MarkdownHeadings {
    fn enter(&self, meta: &HeadingMeta) -> String {
        // let id = meta.content.to_case(convert_case::Case::Kebab);
        let id = self.counter.fetch_add(1, Ordering::SeqCst);
        let id = format!("header-{}", id);

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

fn parser(utf8: &str, item: &str) -> Option<String> {
    let title_index = utf8.find(item);
    let (start, end) = match title_index {
        Some(index) => {
            let start = index + item.len();
            let title_length = utf8.to_string()[start..].find('\"');
            match title_length {
                Some(title_length) => (start, start + title_length),
                None => (0, 0),
            }
        }
        None => (0, 0),
    };

    if end - start > 0 {
        Some(utf8[start..end].to_string())
    } else {
        None
    }
}

enum HighlightColors {
    Green,
    GreenSoft,
    Red,
    RedSoft,
    Teal,
    TealSoft,
    Blue,
    BlueSoft,
    Yellow,
    YellowSoft,
    Orange,
    OrangeSoft,
}

impl HighlightColors {
    fn all() -> [HighlightColors; 12] {
        [
            HighlightColors::Green,
            HighlightColors::GreenSoft,
            HighlightColors::Red,
            HighlightColors::RedSoft,
            HighlightColors::Teal,
            HighlightColors::TealSoft,
            HighlightColors::Blue,
            HighlightColors::BlueSoft,
            HighlightColors::Yellow,
            HighlightColors::YellowSoft,
            HighlightColors::Orange,
            HighlightColors::OrangeSoft,
        ]
    }
}

impl fmt::Display for HighlightColors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HighlightColors::Green => write!(f, "green"),
            HighlightColors::GreenSoft => write!(f, "green-soft"),
            HighlightColors::Red => write!(f, "red"),
            HighlightColors::RedSoft => write!(f, "red-soft"),
            HighlightColors::Teal => write!(f, "teal"),
            HighlightColors::TealSoft => write!(f, "teal-soft"),
            HighlightColors::Blue => write!(f, "blue"),
            HighlightColors::BlueSoft => write!(f, "blue-soft"),
            HighlightColors::Yellow => write!(f, "yellow"),
            HighlightColors::YellowSoft => write!(f, "yellow-soft"),
            HighlightColors::Orange => write!(f, "orange"),
            HighlightColors::OrangeSoft => write!(f, "orange-soft"),
        }
    }
}

struct HighlightLines {}

impl HighlightLines {
    fn get_color(options: &str, color: HighlightColors, hash: &mut HashMap<String, String>) {
        let parse_string = match color {
            HighlightColors::Green => "highlightGreen=\"",
            HighlightColors::GreenSoft => "highlightGreenSoft=\"",
            HighlightColors::Red => "highlightRed=\"",
            HighlightColors::RedSoft => "highlightRedSoft=\"",
            HighlightColors::Teal => "highlightTeal=\"",
            HighlightColors::TealSoft => "highlightTealSoft=\"",
            HighlightColors::Blue => "highlightBlue=\"",
            HighlightColors::BlueSoft => "highlightBlueSoft=\"",
            HighlightColors::Yellow => "highlightYellow=\"",
            HighlightColors::YellowSoft => "highlightYellowSoft=\"",
            HighlightColors::Orange => "highlightOrange=\"",
            HighlightColors::OrangeSoft => "highlightOrangeSoft=\"",
        };

        if let Some(lines) = parser(options, parse_string) {
            let parts = lines.split(',').map(|s| s.to_string());
            for line in parts {
                hash.insert(line, format!("{}", color));
            }
        }
    }
}

#[derive(Debug)]
struct CodeFence<'a> {
    lang: &'a str,
    highlight: HashMap<String, String>,
    line_numbers: bool,
}

impl<'a> From<&str> for CodeFence<'a> {
    fn from(options: &str) -> CodeFence<'a> {
        let lang = if options.starts_with("sql") {
            "sql"
        } else if options.starts_with("bash") {
            "bash"
        } else if options.starts_with("python") {
            "python"
        } else if options.starts_with("javascript") {
            "javascript"
        } else if options.starts_with("postgresql") {
            "postgresql"
        } else if options.starts_with("postgresql-line-nums") {
            "postgresql-line-nums"
        } else if options.starts_with("rust") {
            "rust"
        } else if options.starts_with("json") {
            "json"
        } else {
            "code"
        };

        let mut highlight = HashMap::new();
        for color in HighlightColors::all() {
            HighlightLines::get_color(options, color, &mut highlight);
        }

        CodeFence {
            lang,
            highlight,
            line_numbers: options.contains("lineNumbers"),
        }
    }
}

pub struct SyntaxHighlighter {}

impl SyntaxHighlighterAdapter for SyntaxHighlighter {
    fn highlight(&self, options: Option<&str>, code: &str) -> String {
        let code = if let Some(options) = options {
            let code = code.to_string();
            let options = CodeFence::from(options);

            // Add line highlighting
            let code = code
                .split('\n')
                .enumerate()
                .map(|(index, code)| {
                    format!(
                        r#"<div class="highlight code-line-highlight-{}">{}</div>"#,
                        match options.highlight.get(&(index + 1).to_string()) {
                            Some(color) => color,
                            _ => "none",
                        },
                        code
                    )
                })
                .join("\n");

            code
        } else {
            code.to_string()
        };

        code
    }

    fn build_pre_tag(&self, _attributes: &HashMap<String, String>) -> String {
        String::from("<pre data-controller=\"copy\"><div class=\"code-toolbar\">
                <span data-action=\"click->copy#codeCopy\" class=\"material-symbols-outlined btn-code-toolbar\">content_copy</span>
                <span class=\"material-symbols-outlined btn-code-toolbar\" disabled>link</span>
                <span class=\"material-symbols-outlined btn-code-toolbar\" disabled>edit</span>
            </div>")
    }

    fn build_code_tag(&self, attributes: &HashMap<String, String>) -> String {
        let data = match attributes.get("class") {
            Some(lang) => lang.replace("language-", ""),
            _ => "".to_string(),
        };

        let parsed_data = CodeFence::from(data.as_str());

        // code-block web component uses codemirror to add syntax highlighting
        format!(
            "<code {} language='{}' data-controller=\"code-block\">",
            if parsed_data.line_numbers {
                "class='line-numbers'"
            } else {
                ""
            },
            parsed_data.lang,
        )
    }
}

pub fn options() -> ComrakOptions {
    let mut options = ComrakOptions::default();

    let render_options = ComrakRenderOptions {
        unsafe_: true,
        ..Default::default()
    };

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
fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &mut F) -> Result<()>
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

pub fn iter_mut_all<F>(node: &mut markdown::mdast::Node, f: &mut F) -> Result<()>
where
    F: FnMut(&mut markdown::mdast::Node) -> Result<()>,
{
    let _ = f(node);
    if let Some(children) = node.children_mut() {
        for child in children {
            let _ = iter_mut_all(child, f);
        }
    }

    Ok(())
}

pub fn nest_relative_links(node: &mut markdown::mdast::Node, path: &PathBuf) {
    let _ = iter_mut_all(node, &mut |node| {
        if let markdown::mdast::Node::Link(ref mut link) = node {
            match Url::parse(&link.url) {
                Ok(url) => {
                    if !url.has_host() {
                        let mut url_path = url.path().to_string();
                        let url_path_path = Path::new(&url_path);
                        match url_path_path.extension() {
                            Some(ext) => {
                                if ext.to_str() == Some(".md") {
                                    let base = url_path_path.with_extension("");
                                    url_path = base.into_os_string().into_string().unwrap();
                                }
                            }
                            _ => {
                                warn!("not markdown path: {:?}", path)
                            }
                        }
                        link.url = path.join(url_path).into_os_string().into_string().unwrap();
                    }
                }
                Err(e) => {
                    warn!("could not parse url in markdown: {}", e)
                }
            }
        }

        Ok(())
    });
}

/// Get the title of the article.
///
/// # Arguments
///
/// * `root` - The root node of the document tree.
///
pub fn get_title<'a>(root: &'a AstNode<'a>) -> anyhow::Result<String> {
    let mut title = None;

    iter_nodes(root, &mut |node| {
        if title.is_some() {
            return Ok(false);
        }

        if let NodeValue::Heading(header) = &node.data.borrow().value {
            if header.level == 1 {
                let content = match node.first_child() {
                    Some(child) => child,
                    None => {
                        warn!("markdown heading has no child");
                        return Ok(false);
                    }
                };
                if let NodeValue::Text(text) = &content.data.borrow().value {
                    title = Some(text.to_owned());
                    return Ok(false);
                }
            }
        }

        Ok(true)
    })?;

    let title = match title {
        Some(title) => title,
        None => String::new(),
    };
    Ok(title)
}

/// Get the social sharing image of the article.
///
/// # Arguments
///
/// * `root` - The root node of the document tree.
///
pub fn get_image<'a>(root: &'a AstNode<'a>) -> Option<String> {
    let re = regex::Regex::new(r#"<img src="([^"]*)" alt="([^"]*)""#).unwrap();
    let mut image = None;
    iter_nodes(root, &mut |node| match &node.data.borrow().value {
        NodeValue::HtmlBlock(html) => match re.captures(&html.literal) {
            Some(c) => {
                if &c[2] != "Author" {
                    image = Some(c[1].to_string());
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            None => Ok(true),
        },
        _ => Ok(true),
    })
    .ok()?;
    image
}

/// Wrap tables in container to allow for x-scroll on overflow.
pub fn wrap_tables<'a>(root: &'a AstNode<'a>, arena: &'a Arena<AstNode<'a>>) -> anyhow::Result<()> {
    iter_nodes(root, &mut |node| {
        if let NodeValue::Table(_) = &node.data.borrow().value {
            let open_tag = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                r#"<div class="overflow-auto w-100">"#.to_string(),
            )))));
            let close_tag = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                "</div>".to_string(),
            )))));
            node.insert_before(open_tag);
            node.insert_after(close_tag);
        }

        Ok(true)
    })?;

    Ok(())
}

/// Generate the table of contents for the article.
///
/// # Arguments
///
/// * `root` - The root node of the document tree.
///
pub fn get_toc<'a>(root: &'a AstNode<'a>) -> anyhow::Result<Vec<TocLink>> {
    let mut links = Vec::new();
    let mut header_counter = 0;

    iter_nodes(root, &mut |node| {
        if let NodeValue::Heading(header) = &node.data.borrow().value {
            header_counter += 1;
            if header.level != 1 {
                let sibling = match node.first_child() {
                    Some(child) => child,
                    None => {
                        warn!("markdown heading has no child");
                        return Ok(false);
                    }
                };
                if let NodeValue::Text(text) = &sibling.data.borrow().value {
                    links.push(TocLink::new(text, header_counter - 1).level(header.level));
                    return Ok(false);
                }
            }
        }

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
        NodeValue::Text(text) => {
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

        NodeValue::Code(node) => {
            texts.push(node.literal.to_owned());
            Ok(true)
        }

        NodeValue::CodeBlock(_node) => {
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
            id: crate::utils::random_string(10),
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
                    class=\"nav-link btn btn-tertiary rounded-0 {active}\"
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
        let (class, icon, title) = if utf8.starts_with("!!! info")
            || utf8.starts_with(r#"{% hint style="info" %}"#)
        {
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
        } else if utf8.starts_with("!!! success")
            || utf8.starts_with(r#"{% hint style="success" %}"#)
        {
            ("admonition-success", "check_circle", "Success")
        } else if utf8.starts_with("!!! quote") {
            ("admonition-quote", "format_quote", "Quote")
        } else if utf8.starts_with("!!! bug") {
            ("admonition-bug", "bug_report", "Bug")
        } else if utf8.starts_with("!!! warning")
            || utf8.starts_with(r#"{% hint style="warning" %}"#)
        {
            ("admonition-warning", "warning", "Warning")
        } else if utf8.starts_with("!!! fail") {
            ("admonition-fail", "dangerous", "Fail")
        } else if utf8.starts_with("!!! danger") || utf8.starts_with(r#"{% hint style="danger" %}"#)
        {
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
    line_numbers: Option<String>,
}

impl CodeBlock {
    fn html(&self, html_type: &str) -> Option<String> {
        let line_numbers: bool = match &self.line_numbers {
            Some(val) => match val.as_str() {
                "true" => true,
                _ => false,
            },
            _ => false,
        };

        match html_type {
            "time" => self.time.as_ref().map(|time| {
                format!(
                    r#"
                        <div class="execution-time">
                            <span class="material-symbols-outlined">timer</span>
                            {}
                        </div>
                    "#,
                    time
                )
            }),
            "code" => match &self.title {
                Some(title) => Some(format!(
                    r#"
                    <div class="code-block with-title {}">
                        <div class="title">
                            {}
                        </div>
                "#,
                    if line_numbers { "line-numbers" } else { "" },
                    title
                )),
                None => Some(format!(
                    r#"
                    <div class="code-block {}">
                    "#,
                    if line_numbers { "line-numbers" } else { "" },
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
                None => Some(
                    r#"
                    <div class="results">
                "#
                    .to_string(),
                ),
            },
            _ => None,
        }
    }
}

// Buffer gitbook items with spacing.
pub fn gitbook_preprocess(item: &str) -> String {
    let re = Regex::new(r"[{][%][^{]*[%][}]").unwrap();
    let mut rsp = item.to_string();
    let mut offset = 0;

    re.find_iter(item).for_each(|m| {
        rsp.insert(m.start() + offset, '\n');
        offset = offset + 1;
        rsp.insert(m.start() + offset, '\n');
        offset = offset + 1;
        rsp.insert(m.end() + offset, '\n');
        offset = offset + 1;
        rsp.insert(m.end() + offset, '\n');
        offset = offset + 1;
    });

    return rsp;
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

    // tracks openning tags and holds items to apppend prior to closing
    let mut info_block_close_items: Vec<Option<String>> = vec![];

    iter_nodes(root, &mut |node| {
        match &mut node.data.borrow_mut().value {
            // Strip .md extensions that gitbook includes in page link urls
            &mut NodeValue::Link(ref mut link) => {
                let path = Path::new(link.url.as_str());

                if path.is_relative() {
                    if link.url.ends_with(".md") {
                        for _ in 0..".md".len() {
                            link.url.pop();
                        }
                    }
                }

                Ok(true)
            }

            &mut NodeValue::Text(ref mut text) => {
                // Strip .md extensions that gitbook includes in page link text
                if text.ends_with(".md") {
                    if let Some(parent) = node.parent() {
                        match parent.data.borrow().value {
                            NodeValue::Link(ref _link) => {
                                for _ in 0..".md".len() {
                                    text.pop();
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if text.starts_with("=== \"") {
                    let mut parent = {
                        match node.parent() {
                            Some(parent) => parent,
                            None => node,
                        }
                    };

                    let tab = Tab::new(text.replace("=== ", "").replace('\"', ""));

                    if tabs.is_empty() {
                        let n =
                            arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                                r#"
                                <ul class="nav nav-tabs">
                            "#
                                .to_string(),
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
                        NodeValue::HtmlInline(tabs.last().unwrap().render()),
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
                            NodeValue::HtmlInline("</ul>".to_string()),
                        ))));

                        parent.insert_after(n);
                        parent.detach();

                        parent = n;

                        let n = arena.alloc(Node::new(RefCell::new(Ast::new(
                            NodeValue::HtmlInline(r#"<div class="tab-content">"#.to_string()),
                        ))));

                        parent.insert_after(n);

                        parent = n;

                        for tab in tabs.iter() {
                            let r = arena.alloc(Node::new(RefCell::new(Ast::new(
                                NodeValue::HtmlInline(format!(
                                    r#"
                                                <div
                                                    class="tab-pane {active} pt-2"
                                                    id="tab-{id}"
                                                    role="tabpanel"
                                                    aria-labelledby="tab-{id}">
                                            "#,
                                    active = if tab.active { "show active" } else { "" },
                                    id = tab.id
                                )),
                            ))));

                            for child in tab.children.iter() {
                                r.append(child);
                            }

                            parent.append(r);
                            parent = r;

                            let n = arena.alloc(Node::new(RefCell::new(Ast::new(
                                NodeValue::HtmlInline(r#"</div>"#.to_string()),
                            ))));

                            parent.insert_after(n);
                            parent = n;
                        }

                        parent.insert_after(arena.alloc(Node::new(RefCell::new(Ast::new(
                            NodeValue::HtmlInline(r#"</div>"#.to_string()),
                        )))));

                        tabs.clear();
                        node.detach();
                    }
                } else if text.starts_with("{% tabs %}") {
                    // remove it
                    node.detach();
                } else if text.starts_with("{% endtab %}") {
                    //remove it
                    node.detach()
                } else if text.starts_with("{% tab title=\"") {
                    let mut parent = {
                        match node.parent() {
                            Some(parent) => parent,
                            None => node,
                        }
                    };

                    let tab = Tab::new(text.replace("{% tab title=\"", "").replace("\" %}", ""));

                    if tabs.is_empty() {
                        let n =
                            arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                                r#"
                                <ul class="nav nav-tabs">
                            "#
                                .to_string(),
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
                        NodeValue::HtmlInline(tabs.last().unwrap().render()),
                    )))));

                    // Remove the "===" from the tree.
                    node.detach();
                } else if text.starts_with("{% endtabs %}") {
                    let mut parent = {
                        match node.parent() {
                            Some(parent) => parent,
                            None => node,
                        }
                    };

                    if !tabs.is_empty() {
                        let n = arena.alloc(Node::new(RefCell::new(Ast::new(
                            NodeValue::HtmlInline("</ul>".to_string()),
                        ))));

                        parent.insert_after(n);
                        parent.detach();

                        parent = n;

                        let n = arena.alloc(Node::new(RefCell::new(Ast::new(
                            NodeValue::HtmlInline(r#"<div class="tab-content">"#.to_string()),
                        ))));

                        parent.insert_after(n);

                        parent = n;

                        for tab in tabs.iter() {
                            let r = arena.alloc(Node::new(RefCell::new(Ast::new(
                                NodeValue::HtmlInline(format!(
                                    r#"
                                                <div
                                                    class="tab-pane {active} pt-2"
                                                    id="tab-{id}"
                                                    role="tabpanel"
                                                    aria-labelledby="tab-{id}">
                                            "#,
                                    active = if tab.active { "show active" } else { "" },
                                    id = tab.id
                                )),
                            ))));

                            for child in tab.children.iter() {
                                r.append(child);
                            }

                            parent.append(r);
                            parent = r;

                            let n = arena.alloc(Node::new(RefCell::new(Ast::new(
                                NodeValue::HtmlInline(r#"</div>"#.to_string()),
                            ))));

                            parent.insert_after(n);
                            parent = n;
                        }

                        parent.insert_after(arena.alloc(Node::new(RefCell::new(Ast::new(
                            NodeValue::HtmlInline(r#"</div>"#.to_string()),
                        )))));

                        tabs.clear();
                        node.detach();
                    }
                } else if text.starts_with("!!! info")
                    || text.starts_with(r#"{% hint style="info" %}"#)
                    || text.starts_with("!!! bug")
                    || text.starts_with("!!! tip")
                    || text.starts_with("!!! note")
                    || text.starts_with("!!! abstract")
                    || text.starts_with("!!! example")
                    || text.starts_with("!!! warning")
                    || text.starts_with(r#"{% hint style="warning" %}"#)
                    || text.starts_with("!!! question")
                    || text.starts_with("!!! success")
                    || text.starts_with(r#"{% hint style="success" %}"#)
                    || text.starts_with("!!! quote")
                    || text.starts_with("!!! fail")
                    || text.starts_with("!!! danger")
                    || text.starts_with(r#"{% hint style="danger" %}"#)
                    || text.starts_with("!!! generic")
                {
                    let parent = node.parent().unwrap();

                    let admonition: Admonition = Admonition::from(text.as_ref());

                    let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                        admonition.html(),
                    )))));

                    info_block_close_items.push(None);
                    parent.insert_after(n);
                    parent.detach();
                } else if text.starts_with("!!! code_block") || text.starts_with("{% code ") {
                    let parent = node.parent().unwrap();

                    let title = parser(text.as_ref(), r#"title=""#);
                    let time = parser(text.as_ref(), r#"time=""#);
                    let line_numbers = parser(text.as_ref(), r#"lineNumbers=""#);
                    let code_block = CodeBlock {
                        time,
                        title,
                        line_numbers,
                    };

                    if let Some(html) = code_block.html("code") {
                        let n = arena.alloc(Node::new(RefCell::new(Ast::new(
                            NodeValue::HtmlInline(html),
                        ))));
                        parent.insert_after(n);
                    }

                    // add time to info block to be appended prior to closing
                    info_block_close_items.push(code_block.html("time"));
                    parent.detach();
                } else if text.starts_with("!!! results") {
                    let parent = node.parent().unwrap();

                    let title = parser(text.as_ref(), r#"title=""#);
                    let code_block = CodeBlock {
                        time: None,
                        title,
                        line_numbers: None,
                    };

                    if let Some(html) = code_block.html("results") {
                        let n = arena.alloc(Node::new(RefCell::new(Ast::new(
                            NodeValue::HtmlInline(html),
                        ))));
                        parent.insert_after(n);
                    }

                    info_block_close_items.push(None);
                    parent.detach();
                } else if text.contains("{% content-ref url=") {
                    let url = parser(text.as_ref(), r#"url=""#);

                    let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                        format!(
                            r#"<div>
                                <a href="{}">
                                    <div>"#,
                            url.unwrap(),
                        ),
                    )))));

                    let parent = node.parent().unwrap();

                    info_block_close_items.push(None);
                    parent.insert_after(n);
                    parent.detach();
                } else if (text.starts_with("!!!")
                    || text.starts_with("{% endhint %}")
                    || text.starts_with("{% endcode %}"))
                    && !info_block_close_items.is_empty()
                {
                    let parent = node.parent().unwrap();

                    match info_block_close_items.pop() {
                        Some(html) => match html {
                            Some(html) => {
                                let timing = arena.alloc(Node::new(RefCell::new(Ast::new(
                                    NodeValue::HtmlInline(format!("{html} </div>")),
                                ))));
                                parent.insert_after(timing);
                            }
                            None => {
                                let n = arena.alloc(Node::new(RefCell::new(Ast::new(
                                    NodeValue::HtmlInline(
                                        r#"
                                    </div>
                                    "#
                                        .to_string(),
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
                                    .to_string(),
                                ),
                            ))));

                            parent.insert_after(n);
                        }
                    }

                    parent.detach();
                } else if text.starts_with("{% endcontent-ref %}") {
                    let parent = node.parent().unwrap();

                    let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                        r#"
                                </div>
                            </a>
                        </div>
                        "#
                        .to_string(),
                    )))));

                    parent.insert_after(n);
                    parent.detach()
                }

                // TODO montana
                // *text = text.as_bytes().to_vec();

                Ok(true)
            }

            _ => {
                if !tabs.is_empty() {
                    let last_tab = tabs.last_mut().unwrap();
                    let ancestors = node.ancestors();
                    let mut pushed = false;

                    // Check that we haven't pushed it's parent in yet.
                    for parent in ancestors {
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
        // TODO imrpove this .display().to_string()
        let guides = glob::glob(&config::cms_dir().join("docs/**/*.md").display().to_string())
            .expect("glob failed");
        let blogs = glob::glob(&config::cms_dir().join("blog/**/*.md").display().to_string())
            .expect("glob failed");
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
            Index::create_in_dir(Self::path(), Self::schema())
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
            let title_text = get_title(root).unwrap();
            let body_text = get_text(root).unwrap().into_iter().join(" ");

            let title_field = schema.get_field("title").unwrap();
            let body_field = schema.get_field("body").unwrap();
            let path_field = schema.get_field("path").unwrap();
            let title_regex_field = schema.get_field("title_regex").unwrap();

            info!("found path: {path}", path = path.display());
            let path = path
                .to_str()
                .unwrap()
                .to_string()
                .split("content")
                .last()
                .unwrap()
                .to_string()
                .replace("README", "")
                .replace(&config::cms_dir().display().to_string(), "");
            let mut doc = Document::default();
            doc.add_text(title_field, &title_text);
            doc.add_text(body_field, &body_text);
            doc.add_text(path_field, &path);
            doc.add_text(title_regex_field, &title_text);

            index_writer.add_document(doc)?;
        }

        tokio::task::spawn_blocking(move || -> tantivy::Result<u64> { index_writer.commit() })
            .await
            .unwrap()?;

        Ok(())
    }

    pub fn open() -> tantivy::Result<SearchIndex> {
        let path = Self::path();

        if !path.exists() {
            std::fs::create_dir(&path)
                .expect("failed to create search_index directory, is the filesystem writable?");
        }

        let index = match tantivy::Index::open_in_dir(&path) {
            Ok(index) => index,
            Err(err) => {
                warn!(
                    "Failed to open Tantivy index in '{}', creating an empty one, error: {}",
                    path.display(),
                    err
                );
                Index::create_in_dir(&path, Self::schema())?
            }
        };

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
                .replace(&config::static_dir().display().to_string(), "");

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
                body.split(' ').take(20).collect::<Vec<&str>>().join(" ") + "&nbsp;..."
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::markdown::parser;

    #[test]
    fn parser_title() {
        let to_parse = r#"!!! code_block title="Your Title""#;
        let result = parser(to_parse, r#"title=""#);
        assert_eq!(result, Some("Your Title".to_string()));
    }

    #[test]
    fn parser_time() {
        let to_parse = r#"!!! code_block time="23ms (123.123)""#;
        let result = parser(to_parse, r#"time=""#);
        assert_eq!(result, Some("23ms (123.123)".to_string()));
    }

    #[test]
    fn parser_multiple_flags() {
        let to_parse = r#"!!! code_block title="Your Title" not_real_item="Should not find" time="23ms (123.123)""#;
        let result = parser(to_parse, r#"time=""#);
        assert_eq!(result, Some("23ms (123.123)".to_string()));
    }

    #[test]
    fn parser_none() {
        let to_parse = "!!! code_block";
        let result = parser(to_parse, r#"time=""#);
        assert_eq!(result, None);
    }
}
