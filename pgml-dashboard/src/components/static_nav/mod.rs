use crate::components::StaticNavLink;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, Clone, Default)]
pub struct StaticNav {
    pub links: Vec<StaticNavLink>,
}

impl StaticNav {
    pub fn add_link(&mut self, link: StaticNavLink) {
        self.links.push(link);
    }

    // pub fn get_active(self) -> StaticNavLink {
    //     match self.links.iter().find(|item| item.active) {
    //         Some(item) => item.clone(),
    //         None => StaticNavLink::default(),
    //     }
    // }

    pub fn unique_id(&self) -> String {
        let mut id = String::new();
        for link in &self.links {
            id.push_str(&link.name);
            id.push_str(&link.disabled.to_string());
            id.push_str(&link.href);
        }

        let mut s = DefaultHasher::new();
        id.hash(&mut s);
        format!("nav{}", s.finish().to_string())
    }
}
