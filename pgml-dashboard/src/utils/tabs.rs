use anyhow::anyhow;

pub struct Tab<'a> {
    pub name: &'a str,
    pub content: String,
}

pub struct Tabs<'a> {
    pub tabs: Vec<Tab<'a>>,
    pub default: &'a str,
    pub active: &'a str,
}

impl<'a> Tabs<'a> {
    pub fn new(
        tabs: Vec<Tab<'a>>,
        default: Option<&'a str>,
        active: Option<&'a str>,
    ) -> anyhow::Result<Self> {
        let default = match default {
            Some(default) => default.clone(),
            _ => tabs
                .get(0)
                .ok_or(anyhow!("There must be at least one tab."))?
                .name
                .clone(),
        };

        let active = active
            .and_then(|name| {
                let found = tabs.iter().find(|tab| tab.name == name);
                let just_name = found.map(|tab| tab.name);
                just_name
            })
            .unwrap_or(default.clone());

        Ok(Tabs {
            tabs,
            default,
            active,
        })
    }
}
