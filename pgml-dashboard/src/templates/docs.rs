use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};

use crate::utils::markdown::SearchResult;

/// The search results template.
#[derive(TemplateOnce)]
#[template(path = "components/search.html")]
pub struct Search {
    pub query: String,
    pub results: Vec<SearchResult>,
}

/// Table of contents link.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
