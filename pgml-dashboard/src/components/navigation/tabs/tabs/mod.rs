use crate::components::component;
use crate::components::navigation::tabs::Tab;
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
}

component!(Tabs);
