use pgml_components::component;
use sailfish::TemplateOnce;

use crate::notifications::Notification;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "notifications/marketing/feature_banner/template.html")]
pub struct FeatureBanner {
    pub notification: Option<Notification>,
}

impl FeatureBanner {
    pub fn new() -> FeatureBanner {
        FeatureBanner { notification: None }
    }

    pub fn from_notification(notification: Option<&Notification>) -> FeatureBanner {
        match notification {
            Some(notification) => {
                return FeatureBanner {
                    notification: Some(notification.clone()),
                }
            }
            None => return FeatureBanner { notification: None },
        }
    }
}

component!(FeatureBanner);
