//! Documentation and blog templates.
use sailfish::TemplateOnce;

use crate::utils::markdown::SearchResult;

/// Documentation and blog link used in the left nav.
#[derive(TemplateOnce, Debug, Clone)]
#[template(path = "components/link.html")]
pub struct NavLink {
    pub id: String,
    pub title: String,
    pub href: String,
    pub children: Vec<NavLink>,
    pub open: bool,
}

impl NavLink {
    /// Create a new documentation link.
    pub fn new(title: &str) -> NavLink {
        NavLink {
            id: crate::utils::random_string(25),
            title: title.to_owned(),
            href: "#".to_owned(),
            children: vec![],
            open: false,
        }
    }

    /// Set the link href.
    pub fn href(mut self, href: &str) -> NavLink {
        self.href = href.to_owned();
        self
    }

    /// Set the link's children which are shown when the link is expanded
    /// using Bootstrap's collapse.
    pub fn children(mut self, children: Vec<NavLink>) -> NavLink {
        self.children = children;
        self
    }

    /// Automatically expand the link and it's parents
    /// when one of the children is visible.
    pub fn should_open(&mut self, path: &str) -> bool {
        let open = if self.children.is_empty() {
            self.open = self.href.contains(&path);
            self.open
        } else {
            for child in self.children.iter_mut() {
                if child.should_open(path) {
                    self.open = true;
                }
            }

            self.open
        };

        open
    }
}

/// The search results template.
#[derive(TemplateOnce)]
#[template(path = "components/search.html")]
pub struct Search {
    pub query: String,
    pub results: Vec<SearchResult>,
}

/// Table of contents link.
#[derive(Clone, Debug)]
pub struct TocLink {
    pub title: String,
    pub id: String,
    pub level: u8,
}

impl TocLink {
    /// Creates a new table of contents link.
    ///
    /// # Arguments
    ///
    /// * `title` - The title of the link.
    ///
    pub fn new(title: &str, counter: usize) -> TocLink {
        let id = format!("header-{}", counter);

        TocLink {
            title: title.to_string(),
            id,
            level: 0,
        }
    }

    /// Sets the level of the link.
    ///
    /// The level represents the header level, e.g. h1, h2, h3, h4, etc.
    pub fn level(mut self, level: u8) -> TocLink {
        self.level = level;
        self
    }
}

/// Table of contents template.
#[derive(TemplateOnce)]
#[template(path = "components/toc.html")]
pub struct Toc {
    pub links: Vec<TocLink>,
}
