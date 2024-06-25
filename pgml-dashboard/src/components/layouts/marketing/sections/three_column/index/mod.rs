use sailfish::TemplateOnce;
use pgml_components::{component, Component};

#[derive(TemplateOnce, Default)]
#[template(path = "layouts/marketing/sections/three_column/index/template.html")]
pub struct Index {
    title: Component,
    col_1: Component,
    col_2: Component,
    col_3: Component,
}

impl Index {
    pub fn new() -> Index {
        Index {
            title: "".into(),
            col_1: "".into(),
            col_2: "".into(),
            col_3: "".into(),
        }
    }

    pub fn set_title(mut self, title: Component) -> Self {
        self.title = title;
        self
    }

    pub fn set_col_1(mut self, col_1: Component) -> Self {
        self.col_1 = col_1;
        self
    }

    pub fn set_col_2(mut self, col_2: Component) -> Self {
        self.col_2 = col_2;
        self
    }

    pub fn set_col_3(mut self, col_3: Component) -> Self {
        self.col_3 = col_3;
        self
    }
}

component!(Index);