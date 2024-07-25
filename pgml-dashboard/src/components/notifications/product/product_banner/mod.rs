use crate::{
    notifications::{Notification, NotificationLevel},
    utils::random_string,
};
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "notifications/product/product_banner/template.html")]
pub struct ProductBanner {
    notification: Option<Notification>,
    location_id: String,
    url: String,
    show_modal_on_load: bool,
}

impl ProductBanner {
    pub fn from_notification(notification: Option<&Notification>) -> ProductBanner {
        let mut unique_target = random_string(10);
        unique_target.insert(0, 'a');
        let location_id = ProductBanner::make_location_id(notification.clone(), unique_target.clone());
        let url = ProductBanner::make_url(notification.clone(), unique_target.clone());

        ProductBanner {
            notification: notification.cloned(),
            location_id,
            url,
            show_modal_on_load: true,
        }
    }

    pub fn get_location_id(&self) -> String {
        self.location_id.clone()
    }

    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    pub fn set_show_modal_on_load(mut self, show_modal_on_load: bool) -> ProductBanner {
        self.show_modal_on_load = show_modal_on_load;
        self
    }

    fn make_location_id(notification: Option<&Notification>, random_target: String) -> String {
        match notification {
            Some(notification) => match notification.level {
                NotificationLevel::ProductHigh => random_target,
                _ => {
                    format!(
                        "product-banner{}{}",
                        notification.level.to_string(),
                        notification
                            .deployment
                            .as_ref()
                            .and_then(|id| Some(format!("-{}", id)))
                            .unwrap_or(String::new())
                    )
                }
            },
            _ => random_target,
        }
    }

    fn make_url(notification: Option<&Notification>, random_target: String) -> String {
        let mut url = format!("/dashboard/notifications/product");

        url.push_str(match notification {
            Some(notification) => match notification.level {
                NotificationLevel::ProductHigh => "/remove_banner",
                _ => "/replace_banner",
            },
            None => "/remove_banner",
        });

        let query_params: Vec<Option<String>> = vec![
            notification.and_then(|n| Some(format!("id={}", n.id))),
            notification.and_then(|n| {
                n.deployment
                    .as_ref()
                    .and_then(|id| Some(format!("deployment_id={}", id)))
            }),
            Some(format!("target={}", random_target)),
        ];

        let all_params = query_params
            .iter()
            .filter_map(|x| x.clone())
            .collect::<Vec<String>>()
            .join("&");

        url.push_str(&("?".to_owned() + &all_params));

        url
    }
}

component!(ProductBanner);
