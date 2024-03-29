use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "pagination/template.html")]
pub struct Pagination {
    count: usize,
    default_index: usize,
    timed: bool,
}

impl Pagination {
    pub fn new(count: usize) -> Pagination {
        Pagination {
            count,
            default_index: 0,
            timed: false,
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
