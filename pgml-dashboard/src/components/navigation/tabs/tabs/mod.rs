use crate::components::navigation::tabs::Tab;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default)]
#[template(path = "navigation/tabs/tabs/template.html")]
pub struct Tabs {
    tabs: Vec<Tab>,
}

impl Tabs {
    pub fn new(tabs: &[Tab]) -> Tabs {
        // Set the first tab to active if none are.
        let mut tabs = tabs.to_vec();
        if tabs.iter().all(|t| !t.is_active()) {
            tabs = tabs
                .into_iter()
                .enumerate()
                .map(|(i, tab)| if i == 0 { tab.active() } else { tab })
                .collect();
        }

        Tabs { tabs }
    }

    pub fn active_tab(mut self, name: impl ToString) -> Self {
        let tabs = self
            .tabs
            .into_iter()
            .map(|tab| {
                if tab.name().to_lowercase() == name.to_string().to_lowercase() {
                    tab.active()
                } else {
                    tab.inactive()
                }
            })
            .collect();

        self.tabs = tabs;
        self
    }
}

component!(Tabs);
