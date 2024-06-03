//! Documentation and blog templates.
use sailfish::TemplateOnce;

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
    pub level: i32,
    pub id_suffix: String,
}

impl IndexLink {
    /// Create a new documentation link.
    pub fn new(title: &str, level: i32) -> IndexLink {
        IndexLink {
            id: crate::utils::random_string(25),
            title: title.to_owned(),
            href: "#".to_owned(),
            children: vec![],
            open: false,
            active: false,
            level,
            id_suffix: "".to_owned(),
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
    /// TODO all this str/replace logic should happen once to construct cached versions
    /// that can be more easily checked, during collection construction.
    pub fn should_open(&mut self, path: &std::path::Path) -> &mut Self {
        let path_prefix = path.with_extension("");
        let path_str = path_prefix.to_str().expect("must be a string");
        let suffix = path_str
            .replace(crate::utils::config::cms_dir().to_str().unwrap(), "")
            .replace("README", "");
        if suffix.is_empty() {
            // special case for the index url that would otherwise match everything
            if self.href.is_empty() {
                self.active = true;
                self.open = false;
                return self;
            } else {
                return self;
            }
        }
        self.active = self.href.ends_with(&suffix);
        self.open = self.active;
        for child in self.children.iter_mut() {
            if child.should_open(path).open {
                self.open = true;
            }
        }
        self
    }

    // Adds a suffix to this and all children ids.
    // this prevents id collision with multiple naves on one screen
    // like d-none for mobile nav
    pub fn id_suffix(mut self, id_suffix: &str) -> IndexLink {
        self.id_suffix = id_suffix.to_owned();
        self
    }
}
