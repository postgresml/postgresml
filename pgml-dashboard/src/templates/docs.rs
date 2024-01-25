use convert_case;
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
    /// * `counter` - The number of times that header is in the document
    ///
    pub fn new(title: &str, counter: usize) -> TocLink {
        let conv = convert_case::Converter::new().to_case(convert_case::Case::Kebab);
        let id = conv.convert(title.to_string());

        // gitbook style id's
        let id = format!(
            "{id}{}",
            if counter > 0 {
                format!("-{counter}")
            } else {
                String::new()
            }
        );

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

    /// Converts gitbook link fragment to toc header
    pub fn from_fragment(link: String) -> TocLink {
        match link.is_empty() {
            true => TocLink {
                title: String::new(),
                id: String::new(),
                level: 0,
            },
            _ => TocLink {
                title: link.clone(),
                id: format!("{}", link.clone()),
                level: 0,
            },
        }
    }
}
