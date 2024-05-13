use crate::components::StaticNavLink;

#[derive(Debug, Clone, Default)]
pub struct StaticNav {
    pub links: Vec<StaticNavLink>,
}

impl StaticNav {
    pub fn add_link(&mut self, link: StaticNavLink) {
        self.links.push(link);
    }

    pub fn get_active(self) -> StaticNavLink {
        match self.links.iter().find(|item| item.active) {
            Some(item) => item.clone(),
            None => StaticNavLink::default(),
        }
    }
}
