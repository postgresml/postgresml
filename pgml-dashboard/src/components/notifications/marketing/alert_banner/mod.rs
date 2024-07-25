use pgml_components::component;
use sailfish::TemplateOnce;

use crate::notifications::Notification;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "notifications/marketing/alert_banner/template.html")]
pub struct AlertBanner {
    pub notification: Option<Notification>,
}

impl AlertBanner {
    pub fn new() -> AlertBanner {
        AlertBanner { notification: None }
    }

    pub fn from_notification(notification: Option<&Notification>) -> AlertBanner {
        match notification {
            Some(notification) => {
                return AlertBanner {
                    notification: Some(notification.clone()),
                }
            }
            None => return AlertBanner { notification: None },
        }
    }
}

component!(AlertBanner);
