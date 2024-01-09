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
    pub fn new(tabs: Vec<Tab<'a>>, default: Option<&'a str>, active: Option<&'a str>) -> anyhow::Result<Self> {
        let default = match default {
            Some(default) => default,
            _ => tabs.get(0).ok_or(anyhow!("There must be at least one tab."))?.name,
        };

        let active = active
            .and_then(|name| {
                let found = tabs.iter().find(|tab| tab.name == name);

                found.map(|tab| tab.name)
            })
            .unwrap_or(default);

        Ok(Tabs { tabs, default, active })
    }
}
