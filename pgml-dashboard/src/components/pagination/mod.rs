use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "pagination/template.html")]
pub struct Pagination {
    count: usize,
    timed: bool,
    identifier: u16,
    active_index: Option<usize>,
    clickable: bool,
}

impl Pagination {
    pub fn new(count: usize, identifier: u16) -> Pagination {
        Pagination {
            count,
            timed: false,
            identifier,
            active_index: None,
            clickable: true,
        }
    }

    pub fn timed(mut self) -> Self {
        self.timed = true;
        self
    }

    // When the user wantes to set the active index on render.
    pub fn active_index(mut self, index: usize) -> Self {
        self.active_index = Some(index);
        self
    }

    // Prevents hover states.
    pub fn not_clickable(mut self) -> Self {
        self.clickable = false;
        self
    }
}

component!(Pagination);
