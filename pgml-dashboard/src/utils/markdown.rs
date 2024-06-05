use crate::api::cms::{DocType, Document};
use crate::{templates::docs::TocLink, utils::config};
use anyhow::Context;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use comrak::{
    adapters::{HeadingAdapter, HeadingMeta, SyntaxHighlighterAdapter},
    arena_tree::Node,
    nodes::{Ast, AstNode, NodeValue},
    Arena, ComrakExtensionOptions, ComrakOptions, ComrakRenderOptions,
};
use convert_case;
use itertools::Itertools;
use regex::Regex;
use std::fmt;
use std::sync::Mutex;
use url::Url;

// Excluded paths in the pgml-cms directory
const EXCLUDED_DOCUMENT_PATHS: [&str; 2] = ["blog/README.md", "blog/SUMMARY.md"];

pub struct MarkdownHeadings {
    header_map: Arc<Mutex<HashMap<String, usize>>>,
}

impl Default for MarkdownHeadings {
    fn default() -> Self {
        Self {
            header_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl MarkdownHeadings {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Sets the document headers
///
/// uses toclink to ensure header id matches what the TOC expects
///
impl HeadingAdapter for MarkdownHeadings {
    fn enter(&self, meta: &HeadingMeta) -> String {
        let conv = convert_case::Converter::new().to_case(convert_case::Case::Kebab);
        let id = conv.convert(meta.content.to_string());

        let index = match self.header_map.lock().unwrap().get(&id) {
            Some(value) => value + 1,
            _ => 0,
        };
        self.header_map.lock().unwrap().insert(id.clone(), index);

        let id = TocLink::new(&id, index).id;

        match meta.level {
            1 => format!(r##"<h1 class="h1 mb-5" id="{id}"><a href="#{id}">"##),
            2 => format!(r##"<h2 class="h2 mb-4 mt-5" id="{id}"><a href="#{id}">"##),
            3 => format!(r##"<h3 class="h3 mb-4 mt-5" id="{id}"><a href="#{id}">"##),
            4 => format!(r##"<h4 class="h5 mb-3 mt-3" id="{id}"><a href="#{id}">"##),
            5 => format!(r##"<h5 class="h6 mb-2 mt-4" id="{id}"><a href="#{id}">"##),
            6 => format!(r##"<h6 class="h6 mb-1 mt-1" id="{id}"><a href="#{id}">"##),
            _ => unreachable!(),
        }
    }

    fn exit(&self, meta: &HeadingMeta) -> String {
        match meta.level {
            1 => r#"</a></h1>"#,
            2 => r#"</a></h2>"#,
            3 => r#"</a></h3>"#,
            4 => r#"</a></h4>"#,
            5 => r#"</a></h5>"#,
            6 => r#"</a></h6>"#,
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
        } else if options.starts_with("javascript") || options.eq_ignore_ascii_case("js") {
            "javascript"
        } else if options.starts_with("postgresql") {
            "postgresql"
        } else if options.starts_with("postgresql-line-nums") {
            "postgresql-line-nums"
        } else if options.starts_with("rust") {
            "rust"
        } else if options.starts_with("cpp") {
            "cpp"
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

/// Get the articles author image, name, and publish date.
///
/// # Arguments
///
/// * `root` - The root node of the document tree.
///
pub fn get_author<'a>(root: &'a AstNode<'a>) -> (Option<String>, Option<chrono::NaiveDate>, Option<String>) {
    let re = regex::Regex::new(r#"<img src="([^"]*)" alt="([^"]*)""#).unwrap();
    let mut image = None;
    let mut name = None;
    let mut date = None;
    match iter_nodes(root, &mut |node| match &node.data.borrow().value {
        NodeValue::HtmlBlock(html) => match re.captures(&html.literal) {
            Some(c) => {
                if &c[2] == "Author" {
                    image = Some(c[1].to_string());
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            None => Ok(true),
        },
        // author and name are assumed to be the next two lines of text after the author image.
        NodeValue::Text(text) => {
            if image.is_some() && name.is_none() && date.is_none() {
                name = Some(text.clone());
            } else if image.is_some() && name.is_some() && date.is_none() {
                date = Some(text.clone());
            }
            Ok(true)
        }
        _ => Ok(true),
    }) {
        Ok(_) => {
            let date: Option<chrono::NaiveDate> = match &date {
                Some(date) => {
                    let date_s = date.replace(",", "");
                    let date_v = date_s.split(" ").collect::<Vec<&str>>();
                    let month = date_v[0];
                    match month.parse::<chrono::Month>() {
                        Ok(month) => {
                            let (day, year) = (date_v[1], date_v[2]);
                            let date = format!("{}-{}-{}", month.number_from_month(), day, year);
                            chrono::NaiveDate::parse_from_str(&date, "%m-%e-%Y").ok()
                        }
                        _ => None,
                    }
                }
                _ => None,
            };

            // if date is not the correct form assume the date and author did not get parsed correctly.
            if date.is_none() {
                (None, None, image)
            } else {
                (name, date, image)
            }
        }
        _ => (None, None, None),
    }
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
    let mut header_count: HashMap<String, usize> = HashMap::new();

    iter_nodes(root, &mut |node| {
        if let NodeValue::Heading(header) = &node.data.borrow().value {
            if header.level != 1 {
                let sibling = match node.first_child() {
                    Some(child) => child,
                    None => {
                        warn!("markdown heading has no child");
                        return Ok(false);
                    }
                };

                let text = if let NodeValue::Text(text) = &sibling.data.borrow().value {
                    Some(text.clone())
                } else if let NodeValue::Link(_link) = &sibling.data.borrow().value {
                    let text = sibling
                        .children()
                        .into_iter()
                        .map(|child| {
                            if let NodeValue::Text(text) = &child.data.borrow().value {
                                text.clone()
                            } else {
                                "".to_string()
                            }
                        })
                        .join("");
                    Some(text)
                } else {
                    None
                };

                if let Some(text) = text {
                    let index = match header_count.get(&text) {
                        Some(index) => index + 1,
                        _ => 0,
                    };

                    header_count.insert(text.clone(), index);

                    links.push(TocLink::new(&text, index).level(header.level));
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
        let (class, icon, title) = if utf8.starts_with("!!! info") || utf8.starts_with(r#"{% hint style="info" %}"#) {
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
        } else if utf8.starts_with("!!! success") || utf8.starts_with(r#"{% hint style="success" %}"#) {
            ("admonition-success", "check_circle", "Success")
        } else if utf8.starts_with("!!! quote") {
            ("admonition-quote", "format_quote", "Quote")
        } else if utf8.starts_with("!!! bug") {
            ("admonition-bug", "bug_report", "Bug")
        } else if utf8.starts_with("!!! warning") || utf8.starts_with(r#"{% hint style="warning" %}"#) {
            ("admonition-warning", "warning", "Warning")
        } else if utf8.starts_with("!!! fail") {
            ("admonition-fail", "dangerous", "Fail")
        } else if utf8.starts_with("!!! danger") || utf8.starts_with(r#"{% hint style="danger" %}"#) {
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
            // Strip .md extensions that gitbook includes in page link urls.
            &mut NodeValue::Link(ref mut link) => {
                let url = Url::parse(link.url.as_str());

                // Ignore absolute urls that are not site domain, github has .md endpoints
                if url.is_err()
                    || url?.host_str().unwrap_or_else(|| "")
                        == Url::parse(&config::site_domain())?
                            .host_str()
                            .unwrap_or_else(|| "postgresml.org")
                {
                    let fragment = match link.url.find("#") {
                        Some(index) => link.url[index + 1..link.url.len()].to_string(),
                        _ => "".to_string(),
                    };

                    // Remove fragment and the fragment identifier #.
                    for _ in 0..fragment.len()
                        + match fragment.len() {
                            0 => 0,
                            _ => 1,
                        }
                    {
                        link.url.pop();
                    }

                    // Remove file path to make this a relative url.
                    if link.url.ends_with(".md") {
                        for _ in 0..".md".len() {
                            link.url.pop();
                        }
                    }

                    // Reappend the path fragment.
                    let header_id = TocLink::from_fragment(fragment).id;
                    if header_id.len() > 0 {
                        link.url.push('#');
                        for c in header_id.chars() {
                            link.url.push(c)
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
                        let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
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

                    parent.insert_after(arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                        tabs.last().unwrap().render(),
                    ))))));

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
                        let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                            "</ul>".to_string(),
                        )))));

                        parent.insert_after(n);
                        parent.detach();

                        parent = n;

                        let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                            r#"<div class="tab-content">"#.to_string(),
                        )))));

                        parent.insert_after(n);

                        parent = n;

                        for tab in tabs.iter() {
                            let r = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(format!(
                                r#"
                                                <div
                                                    class="tab-pane {active} pt-2"
                                                    id="tab-{id}"
                                                    role="tabpanel"
                                                    aria-labelledby="tab-{id}">
                                            "#,
                                active = if tab.active { "show active" } else { "" },
                                id = tab.id
                            ))))));

                            for child in tab.children.iter() {
                                r.append(child);
                            }

                            parent.append(r);
                            parent = r;

                            let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                                r#"</div>"#.to_string(),
                            )))));

                            parent.insert_after(n);
                            parent = n;
                        }

                        parent.insert_after(arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                            r#"</div>"#.to_string(),
                        ))))));

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
                        let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
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

                    parent.insert_after(arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                        tabs.last().unwrap().render(),
                    ))))));

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
                        let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                            "</ul>".to_string(),
                        )))));

                        parent.insert_after(n);
                        parent.detach();

                        parent = n;

                        let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                            r#"<div class="tab-content">"#.to_string(),
                        )))));

                        parent.insert_after(n);

                        parent = n;

                        for tab in tabs.iter() {
                            let r = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(format!(
                                r#"
                                                <div
                                                    class="tab-pane {active} pt-2"
                                                    id="tab-{id}"
                                                    role="tabpanel"
                                                    aria-labelledby="tab-{id}">
                                            "#,
                                active = if tab.active { "show active" } else { "" },
                                id = tab.id
                            ))))));

                            for child in tab.children.iter() {
                                r.append(child);
                            }

                            parent.append(r);
                            parent = r;

                            let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                                r#"</div>"#.to_string(),
                            )))));

                            parent.insert_after(n);
                            parent = n;
                        }

                        parent.insert_after(arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                            r#"</div>"#.to_string(),
                        ))))));

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
                        let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(html)))));
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
                        let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(html)))));
                        parent.insert_after(n);
                    }

                    info_block_close_items.push(None);
                    parent.detach();
                } else if text.contains("{% content-ref url=") {
                    let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(format!(
                        r#"<div>"#,
                    ))))));
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
                                let timing = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                                    format!("{html} </div>"),
                                )))));
                                parent.insert_after(timing);
                            }
                            None => {
                                let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                                    r#"
                                    </div>
                                    "#
                                    .to_string(),
                                )))));

                                parent.insert_after(n);
                            }
                        },
                        None => {
                            let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                                r#"
                            </div>
                            "#
                                .to_string(),
                            )))));

                            parent.insert_after(n);
                        }
                    }

                    parent.detach();
                } else if text.starts_with("{% endcontent-ref %}") {
                    let parent = node.parent().unwrap();
                    let n = arena.alloc(Node::new(RefCell::new(Ast::new(NodeValue::HtmlInline(
                        r#"</div>"#.to_string(),
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
    pub path: String,
    pub snippet: String,
}

#[derive(Clone)]
pub struct SiteSearch {
    collection: pgml::Collection,
    pipeline: pgml::Pipeline,
}

impl SiteSearch {
    pub async fn new() -> anyhow::Result<Self> {
        let collection = pgml::Collection::new(
            &format!("{}-1", env!("CMS_HASH")),
            Some(
                std::env::var("SITE_SEARCH_DATABASE_URL")
                    .context("Please set the `SITE_SEARCH_DATABASE_URL` environment variable")?,
            ),
        )?;
        let pipeline = pgml::Pipeline::new(
            "hypercloud-site-search-p-0",
            Some(
                serde_json::json!({
                    "title": {
                        "full_text_search": {
                            "configuration": "english"
                        },
                        "semantic_search": {
                            "model": "mixedbread-ai/mxbai-embed-large-v1",
                        }
                    },
                    "contents": {
                        "splitter": {
                            "model": "recursive_character"
                        },
                        "full_text_search": {
                            "configuration": "english"
                        },
                        "semantic_search": {
                            "model": "mixedbread-ai/mxbai-embed-large-v1",
                        }
                    }
                })
                .into(),
            ),
        )?;
        Ok(Self { collection, pipeline })
    }

    pub fn documents() -> Vec<PathBuf> {
        // TODO improve this .display().to_string()
        let guides = glob::glob(&config::cms_dir().join("docs/**/*.md").display().to_string()).expect("glob failed");
        let blogs = glob::glob(&config::cms_dir().join("blog/**/*.md").display().to_string()).expect("glob failed");
        guides
            .chain(blogs)
            .map(|path| path.expect("glob path failed"))
            .collect()
    }

    pub async fn search(
        &self,
        query: &str,
        doc_type: Option<DocType>,
        doc_tags: Option<Vec<String>>,
    ) -> anyhow::Result<Vec<Document>> {
        let mut search = serde_json::json!({
            "query": {
                // "full_text_search": {
                //     "title": {
                //         "query": query,
                //         "boost": 4.0
                //     },
                //     "contents": {
                //         "query": query
                //     }
                // },
                "semantic_search": {
                    "title": {
                        "query": query,
                        "parameters": {
                            "prompt": "Represent this sentence for searching relevant passages: "
                        },
                        "boost": 10.0
                    },
                    "contents": {
                        "query": query,
                        "parameters": {
                            "prompt": "Represent this sentence for searching relevant passages: "
                        },
                        "boost": 1.0
                    }
                }
            },
            "limit": 10
        });
        search["query"]["filter"]["$and"] = serde_json::json!({});
        if let Some(doc_type) = doc_type {
            search["query"]["filter"]["$and"]["doc_type"] = serde_json::json!({
                "$eq": doc_type
            });
        }
        if let Some(doc_tags) = doc_tags {
            search["query"]["filter"]["$and"]["tags"] = serde_json::json!({
                "$in": doc_tags
            });
        }
        let results = self.collection.search_local(search.into(), &self.pipeline).await?;

        results["results"]
            .as_array()
            .context("Error getting results from search")?
            .iter()
            .map(|r| {
                let document: Document = serde_json::from_value(r["document"].clone())?;
                Ok(document)
            })
            .collect()
    }

    pub async fn build(&mut self) -> anyhow::Result<()> {
        self.collection.add_pipeline(&mut self.pipeline).await?;
        let documents: Vec<Document> = futures::future::try_join_all(
            Self::get_document_paths()?
                .into_iter()
                .map(|path| async move { Document::from_path(&path).await }),
        )
        .await?;
        // Filter out documents who only have 1 line (this is usually just an empty document with the title as the first line)
        // and documents that are in our excluded paths list
        let documents: Vec<Document> = documents
            .into_iter()
            .filter(|f| {
                if f.ignore() {
                    return false;
                }

                !EXCLUDED_DOCUMENT_PATHS
                    .iter()
                    .any(|p| f.path == config::cms_dir().join(p))
                    && !f
                        .contents
                        .lines()
                        .skip(1)
                        .collect::<Vec<&str>>()
                        .join("")
                        .trim()
                        .is_empty()
            })
            .collect();
        let documents: Vec<pgml::types::Json> = documents
            .into_iter()
            .map(|d| {
                let mut document_json = serde_json::to_value(d).unwrap();
                document_json["id"] = document_json["path"].clone();
                document_json["path"] = serde_json::json!(document_json["path"]
                    .as_str()
                    .unwrap()
                    .split("content")
                    .last()
                    .unwrap()
                    .to_string()
                    .replace("README", "")
                    .replace(&config::cms_dir().display().to_string(), ""));
                document_json.into()
            })
            .collect();
        self.collection.upsert_documents(documents, None).await
    }

    fn get_document_paths() -> anyhow::Result<Vec<PathBuf>> {
        // TODO imrpove this .display().to_string()
        let guides = glob::glob(&config::cms_dir().join("docs/**/*.md").display().to_string())?;
        let blogs = glob::glob(&config::cms_dir().join("blog/**/*.md").display().to_string())?;
        Ok(guides
            .chain(blogs)
            .map(|path| path.expect("glob path failed"))
            .collect())
    }
}

#[cfg(test)]
mod test {
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
