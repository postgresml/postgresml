use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "pagination/template.html")]
pub struct Pagination {
    count: usize,
    timed: bool,
    identifier: u16,
}

impl Pagination {
    pub fn new(count: usize, identifier: u16) -> Pagination {
        Pagination {
            count,
            timed: false,
            identifier: identifier,
        }
    }

    pub fn timed(mut self) -> Self {
        self.timed = true;
        self
    }
}

component!(Pagination);
