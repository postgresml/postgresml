use crate::{Notification, NotificationLevel};
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "notifications/banner/template.html")]
pub struct Banner {
    pub notification: Notification,
    pub remove_banner: bool,
}

impl Banner {
    pub fn new() -> Banner {
        Banner {
            notification: Notification::default(),
            remove_banner: false,
        }
    }

    pub fn from_notification(notification: Notification) -> Banner {
        Banner {
            notification,
            remove_banner: false,
        }
    }

    pub fn remove_banner(mut self, remove_banner: bool) -> Banner {
        self.remove_banner = remove_banner;
        self
    }
}

component!(Banner);
