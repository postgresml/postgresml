use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "pagination/template.html")]
pub struct Pagination {
    count: usize,
    default_index: usize,
    timed: bool,
    identifier: u16,
}

impl Pagination {
    pub fn new(count: usize, identifier: u16) -> Pagination {
        Pagination {
            count,
            default_index: 0,
            timed: false,
            identifier: identifier,
        }
    }

    pub fn default_index(mut self, default_index: usize) -> Self {
        self.default_index = default_index;
        self
    }

    pub fn timed(mut self) -> Self {
        self.timed = true;
        self
    }
}

component!(Pagination);
