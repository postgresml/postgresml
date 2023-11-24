//! Documentation and blog templates.
use sailfish::TemplateOnce;

use crate::utils::markdown::SearchResult;

/// Documentation and blog link used in the left nav.
#[derive(TemplateOnce, Debug, Clone)]
#[template(path = "cms/index_link/template.html")]
pub struct IndexLink {
    pub id: String,
    pub title: String,
    pub href: String,
    pub children: Vec<IndexLink>,
    pub open: bool,
    pub active: bool,
}

impl IndexLink {
    /// Create a new documentation link.
    pub fn new(title: &str) -> IndexLink {
        IndexLink {
            id: crate::utils::random_string(25),
            title: title.to_owned(),
            href: "#".to_owned(),
            children: vec![],
            open: false,
            active: false,
        }
    }

    /// Set the link href.
    pub fn href(mut self, href: &str) -> IndexLink {
        self.href = href.to_owned();
        self
    }

    /// Set the link's children which are shown when the link is expanded
    /// using Bootstrap's collapse.
    pub fn children(mut self, children: Vec<IndexLink>) -> IndexLink {
        self.children = children;
        self
    }

    /// Automatically expand the link and it's parents
    /// when one of the children is visible.
    pub fn should_open(&mut self, path: &str, root: &std::path::Path) -> &mut Self {
        info!(
            "should_open self: {:?}, path: {:?}, root: {:?}",
            self, path, root
        );
        // if path.is_empty() {
        //     if self.href.as_str() == root.as_os_str() {
        //         return true;
        //     } else {
        //         return false;
        //     }
        // }
        self.active = self.href.ends_with(&path);
        self.open = self.active;
        for child in self.children.iter_mut() {
            if child.should_open(path, root).open {
                self.open = true;
            }
        }
        self
    }
}
