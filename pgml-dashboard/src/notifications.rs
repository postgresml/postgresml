use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, Clone, Default)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub id: String,
    pub dismissible: bool,
    pub viewed: bool,
    pub link: Option<String>,
    pub deployment: Option<String>,
    pub preset_icon: bool,
    pub title: Option<String>,
    pub modal_show_interval: i64,
    pub notification_show_interval: i64,
    pub trigger_modal: bool,
}
impl Notification {
    pub fn new(message: &str) -> Notification {
        let mut s = DefaultHasher::new();
        message.hash(&mut s);

        Notification {
            message: message.to_string(),
            level: NotificationLevel::Level1,
            id: s.finish().to_string(),
            dismissible: true,
            viewed: false,
            link: None,
            deployment: None,
            preset_icon: false,
            title: None,
            modal_show_interval: 90,        // If modal dismissed, show again in 90 days.
            notification_show_interval: 90, // If notification dismissed, show again in 90 days.
            trigger_modal: false,
        }
    }

    pub fn set_level(mut self, level: &NotificationLevel) -> Notification {
        self.level = level.clone();
        self
    }

    pub fn set_dismissible(mut self, dismissible: bool) -> Notification {
        self.dismissible = dismissible;
        self
    }

    pub fn set_link(mut self, link: &str) -> Notification {
        self.link = Some(link.into());
        self
    }

    pub fn set_viewed(mut self, viewed: bool) -> Notification {
        self.viewed = viewed;
        self
    }

    pub fn set_deployment(mut self, deployment: &str) -> Notification {
        self.deployment = Some(deployment.into());
        self
    }

    pub fn has_preset_icon(mut self, show_icon: bool) -> Notification {
        self.preset_icon = show_icon;
        self
    }

    pub fn set_title(mut self, title: &str) -> Notification {
        self.title = Some(title.into());
        self
    }

    pub fn set_modal_show_interval(mut self, interval: i64) -> Notification {
        self.modal_show_interval = interval;
        self
    }

    pub fn set_notification_show_interval(mut self, interval: i64) -> Notification {
        self.notification_show_interval = interval;
        self
    }

    pub fn set_trigger_modal(mut self, trigger_modal: bool) -> Notification {
        self.trigger_modal = trigger_modal;
        self
    }

    pub fn is_alert(level: &NotificationLevel) -> bool {
        match level {
            NotificationLevel::Level1 | NotificationLevel::Level2 | NotificationLevel::Level3 => true,
            _ => false,
        }
    }

    pub fn is_feature(level: &NotificationLevel) -> bool {
        match level {
            NotificationLevel::Feature1 | NotificationLevel::Feature2 | NotificationLevel::Feature3 => true,
            _ => false,
        }
    }

    pub fn next_alert(context: Option<&crate::guards::Cluster>) -> Option<&Notification> {
        match context.as_ref() {
            Some(context) => match &context.notifications {
                Some(notifications) => {
                    match notifications
                        .into_iter()
                        .filter(|n| Notification::is_alert(&n.level))
                        .next()
                    {
                        Some(notification) => return Some(notification),
                        None => return None,
                    }
                }
                None => return None,
            },
            None => return None,
        };
    }

    pub fn next_feature(context: Option<&crate::guards::Cluster>) -> Option<&Notification> {
        match context.as_ref() {
            Some(context) => match &context.notifications {
                Some(notifications) => {
                    match notifications
                        .into_iter()
                        .filter(|n| Notification::is_feature(&n.level))
                        .next()
                    {
                        Some(notification) => return Some(notification),
                        None => return None,
                    }
                }
                None => return None,
            },
            None => return None,
        };
    }

    pub fn next_product_of_level(
        context: &crate::guards::Cluster,
        desired_level: NotificationLevel,
    ) -> Option<&Notification> {
        match &context.notifications {
            Some(notifications) => {
                match notifications
                    .into_iter()
                    .filter(|n| {
                        Notification::product_filter(
                            n,
                            desired_level.clone(),
                            Some(context.context.cluster.id.clone().to_string()),
                        )
                    })
                    .next()
                {
                    Some(notification) => return Some(notification),
                    None => return None,
                }
            }
            None => return None,
        }
    }

    // Determine if product notification matches desired level and deployment id.
    pub fn product_filter(
        notification: &Notification,
        desired_level: NotificationLevel,
        deployment_id: Option<String>,
    ) -> bool {
        match notification.level {
            NotificationLevel::ProductHigh => notification.level == desired_level && notification.viewed == false,
            NotificationLevel::ProductMedium => {
                notification.level == desired_level
                    && notification.deployment == deployment_id
                    && notification.viewed == false
            }
            NotificationLevel::ProductMarketing => notification.level == desired_level && notification.viewed == false,
            _ => false,
        }
    }
}

impl std::fmt::Display for NotificationLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationLevel::Level1 => write!(f, "level1"),
            NotificationLevel::Level2 => write!(f, "level2"),
            NotificationLevel::Level3 => write!(f, "level3"),
            NotificationLevel::Feature1 => write!(f, "feature1"),
            NotificationLevel::Feature2 => write!(f, "feature2"),
            NotificationLevel::Feature3 => write!(f, "feature3"),
            NotificationLevel::ProductHigh => write!(f, "product_high"),
            NotificationLevel::ProductMedium => write!(f, "product_medium"),
            NotificationLevel::ProductMarketing => write!(f, "product_marketing"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum NotificationLevel {
    #[default]
    // global
    Level1,
    Level2,
    Level3,
    // marketing
    Feature1,
    Feature2,
    Feature3,
    // product
    ProductHigh,
    ProductMedium,
    ProductMarketing,
}
