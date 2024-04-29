use pgml_components::{component, Component};
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "inputs/labels/with_tooltip/template.html")]
pub struct WithTooltip {
    component: Component,
    tooltip: String,
    icon: String,
    html: bool,
}

impl WithTooltip {
    pub fn new(component: Component) -> WithTooltip {
        WithTooltip {
            component,
            tooltip: String::new(),
            icon: "info".to_string(),
            html: false,
        }
    }

    pub fn tooltip(mut self, tooltip: impl ToString) -> Self {
        self.tooltip = tooltip.to_string();
        self
    }

    pub fn tooltip_text(self, tooltip: impl ToString) -> Self {
        self.tooltip(tooltip)
    }

    pub fn tooltip_html(mut self, tooltip: impl ToString) -> Self {
        self.tooltip = tooltip.to_string();
        self.html = true;
        self
    }

    pub fn icon(mut self, icon: impl ToString) -> Self {
        self.icon = icon.to_string();
        self
    }
}

component!(WithTooltip);
