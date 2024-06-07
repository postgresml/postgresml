use crate::Notification;
use pgml_components::component;
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "notifications/product/product_banner/template.html")]
pub struct ProductBanner {
    pub notification: Option<Notification>,
    pub location_id: String,
    pub url: String,
}

impl ProductBanner {
    pub fn from_notification(notification: Option<&Notification>) -> ProductBanner {
        let mut location_id = format!("product-banner");
        let mut url = format!("/dashboard/notifications/product/remove_banner");
        if notification.is_some() {
            let notification = notification.clone().unwrap();
            location_id.push_str(&format!("-{}", notification.level.to_string()));
            url.push_str(&format!("?id={}", notification.id));

            if notification.deployment.is_some() {
                let deployment = notification.deployment.clone().unwrap();
                location_id.push_str(&format!("-{}", deployment));
                url.push_str(&format!("&deployment_id={}", deployment));
            }
        }

        match notification {
            Some(notification) => {
                return ProductBanner {
                    notification: Some(notification.clone()),
                    location_id,
                    url,
                }
            }
            None => {
                return ProductBanner {
                    notification: None,
                    location_id,
                    url,
                }
            }
        }
    }

    pub fn get_location_id(&self) -> String {
        self.location_id.clone()
    }

    pub fn get_url(&self) -> String {
        self.url.clone()
    }
}

component!(ProductBanner);
