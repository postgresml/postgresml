use crate::Notification;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "notifications/marketing/feature_banner/template.html")]
pub struct FeatureBanner {
    pub notification: Notification,
    pub remove_banner: bool,
}

impl FeatureBanner {
    pub fn new() -> FeatureBanner {
        FeatureBanner {
            notification: Notification::default(),
            remove_banner: false,
        }
    }

    pub fn from_notification(notification: Option<Notification>) -> FeatureBanner {
        match notification {
            Some(notification) => {
                return FeatureBanner {
                    notification,
                    remove_banner: false,
                }
            }
            None => {
                return FeatureBanner {
                    notification: Notification::default(),
                    remove_banner: true,
                }
            }
        }
    }

    pub fn remove_banner(mut self, remove_banner: bool) -> FeatureBanner {
        self.remove_banner = remove_banner;
        self
    }
}

component!(FeatureBanner);
