#![allow(renamed_and_removed_lints)]

#[macro_use]
extern crate rocket;

use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::route::Route;
use sailfish::TemplateOnce;
use sqlx::PgPool;

pub mod api;
pub mod components;
pub mod fairings;
pub mod forms;
pub mod guards;
pub mod models;
pub mod responses;
pub mod templates;
pub mod types;
pub mod utils;

use components::notifications::marketing::{AlertBanner, FeatureBanner};
use components::notifications::product::ProductBanner;
use guards::Cluster;
use responses::{Error, Response, ResponseOk};
use templates::{components::StaticNav, *};

use crate::components::tables::serverless_models::{ServerlessModels, ServerlessModelsTurbo};
use crate::components::tables::serverless_pricing::{ServerlessPricing, ServerlessPricingTurbo};
use crate::utils::cookies::{NotificationCookie, Notifications};
use crate::utils::urls;
use chrono;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use pgml_components::Component;

#[derive(Debug, Default, Clone)]
pub struct ClustersSettings {
    pub max_connections: u32,
    pub idle_timeout: u64,
    pub min_connections: u32,
}

/// This struct contains information specific to the cluster being displayed in the dashboard.
///
/// The dashboard is built to manage multiple clusters, but the server itself by design is stateless.
/// This gives it a bit of shared state that allows the dashboard to display cluster-specific information.
#[derive(Debug, Default, Clone)]
pub struct Context {
    pub user: models::User,
    pub cluster: models::Cluster,
    pub dropdown_nav: StaticNav,
    pub product_left_nav: StaticNav,
    pub marketing_footer: String,
    pub head_items: Option<String>,
    pub body_components: Vec<Component>,
}

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

#[get("/serverless_models/turboframe?<style>")]
pub fn serverless_models_turboframe(style: String) -> ResponseOk {
    let comp = ServerlessModels::new().set_style_type(&style);
    ResponseOk(ServerlessModelsTurbo::new(comp.into()).render_once().unwrap())
}

#[get("/serverless_pricing/turboframe?<style>")]
pub fn serverless_pricing_turboframe(style: String) -> ResponseOk {
    let comp = ServerlessPricing::new().set_style_type(&style);
    ResponseOk(ServerlessPricingTurbo::new(comp.into()).render_once().unwrap())
}

// Reroute old style query style dashboard links.
#[get("/?<tab>&<id>")]
pub async fn dashboard(tab: Option<&str>, id: Option<i64>) -> Redirect {
    let tab = tab.unwrap_or("Notebooks");

    match tab {
        "Notebooks" => Redirect::to(urls::deployment_notebooks()),

        "Notebook" => match id {
            Some(id) => Redirect::to(urls::deployment_notebook_by_id(id)),
            None => Redirect::to(urls::deployment_notebooks()),
        },

        "Projects" => Redirect::to(urls::deployment_projects()),

        "Project" => match id {
            Some(id) => Redirect::to(urls::deployment_project_by_id(id)),
            None => Redirect::to(urls::deployment_projects()),
        },

        "Models" => Redirect::to(urls::deployment_models()),

        "Model" => match id {
            Some(id) => Redirect::to(urls::deployment_model_by_id(id)),
            None => Redirect::to(urls::deployment_models()),
        },

        "Snapshots" => Redirect::to(urls::deployment_snapshots()),

        "Snapshot" => match id {
            Some(id) => Redirect::to(urls::deployment_snapshot_by_id(id)),
            None => Redirect::to(urls::deployment_snapshots()),
        },

        "Upload_Data" => Redirect::to(urls::deployment_uploader()),
        _ => Redirect::to(urls::deployment_notebooks()),
    }
}

#[get("/playground")]
pub async fn playground(cluster: &Cluster) -> Result<ResponseOk, Error> {
    let mut layout = crate::templates::WebAppBase::new("Playground", &cluster);
    Ok(ResponseOk(layout.render(templates::Playground {})))
}

// Remove Alert and Feature banners after user exits out of the message.
#[get("/notifications/remove_banner?<id>&<notification_type>")]
pub fn remove_banner(id: String, notification_type: String, cookies: &CookieJar<'_>, context: &Cluster) -> ResponseOk {
    let mut viewed = Notifications::get_viewed(cookies);

    viewed.push(NotificationCookie {
        id: id.clone(),
        time_viewed: Some(chrono::Utc::now()),
        time_modal_viewed: None,
    });
    Notifications::update_viewed(&viewed, cookies);

    let notification = match context.notifications.as_ref() {
        Some(notifications) => {
            if notification_type == "alert" {
                notifications
                    .into_iter()
                    .filter(|n: &&Notification| -> bool {
                        Notification::is_alert(&n.level)
                            && !viewed
                                .clone()
                                .into_iter()
                                .map(|x| x.id)
                                .collect::<Vec<String>>()
                                .contains(&n.id)
                    })
                    .next()
            } else if notification_type == "feature" {
                notifications
                    .into_iter()
                    .filter(|n: &&Notification| -> bool {
                        Notification::is_feature(&n.level)
                            && !viewed
                                .clone()
                                .into_iter()
                                .map(|x| x.id)
                                .collect::<Vec<String>>()
                                .contains(&n.id)
                    })
                    .next()
            } else {
                None
            }
        }
        _ => None,
    };

    if notification_type == "alert" {
        return ResponseOk(AlertBanner::from_notification(notification).render_once().unwrap());
    } else {
        return ResponseOk(FeatureBanner::from_notification(notification).render_once().unwrap());
    }
}

// Replace a product banner after user exits out of the message.
#[get("/notifications/product/replace_banner?<id>&<deployment_id>")]
pub fn replace_banner_product(
    id: String,
    deployment_id: Option<String>,
    cookies: &CookieJar<'_>,
    context: &Cluster,
) -> Result<Response, Error> {
    let mut all_notification_cookies = Notifications::get_viewed(cookies);
    let current_notification_cookie = all_notification_cookies.iter().position(|x| x.id == id);

    match current_notification_cookie {
        Some(index) => {
            all_notification_cookies[index].time_viewed = Some(chrono::Utc::now());
        }
        None => {
            all_notification_cookies.push(NotificationCookie {
                id: id.clone(),
                time_viewed: Some(chrono::Utc::now()),
                time_modal_viewed: None,
            });
        }
    }

    Notifications::update_viewed(&all_notification_cookies, cookies);

    let last_notification: Option<Notification> = context
        .notifications
        .as_ref()
        .unwrap_or(&vec![] as &Vec<Notification>)
        .clone()
        .into_iter()
        .find(|n: &Notification| -> bool { n.id == id });

    let next_notification = match context.notifications.as_ref() {
        Some(notifications) => notifications
            .clone()
            .into_iter()
            .filter(|n: &Notification| -> bool {
                let n = n.clone().set_viewed(n.id == id);
                if last_notification.clone().is_none() {
                    return false;
                } else {
                    Notification::product_filter(
                        &n,
                        last_notification.clone().unwrap().level.clone(),
                        deployment_id.clone(),
                    )
                }
            })
            .next(),
        _ => None,
    };

    let component = ProductBanner::from_notification(next_notification.as_ref());
    let target = ProductBanner::from_notification(last_notification.as_ref()).get_location_id();
    let content = component.render_once().unwrap();
    let turbo_stream = format!(
        r##"<turbo-stream action="replace" targets=".{}">
<template>
{}
</template>
</turbo-stream>"##,
        target, content
    );
    return Ok(Response::turbo_stream(turbo_stream));
}

// Remove a product banners after user exits out of the message.
#[get("/notifications/product/remove_banner?<id>&<target>")]
pub fn remove_banner_product(id: String, target: String, cookies: &CookieJar<'_>) -> Result<Response, Error> {
    let mut all_notification_cookies = Notifications::get_viewed(cookies);

    let current_notification_cookie = all_notification_cookies.iter().position(|x| x.id == id);

    match current_notification_cookie {
        Some(index) => {
            all_notification_cookies[index].time_viewed = Some(chrono::Utc::now());
        }
        None => {
            all_notification_cookies.push(NotificationCookie {
                id: id.clone(),
                time_viewed: Some(chrono::Utc::now()),
                time_modal_viewed: None,
            });
        }
    }

    Notifications::update_viewed(&all_notification_cookies, cookies);

    let turbo_stream = format!(
        r##"<turbo-stream action="remove" targets=".{}">
<template>
</template>
</turbo-stream>"##,
        target
    );
    return Ok(Response::turbo_stream(turbo_stream));
}

// Update cookie to show the user has viewed the modal.
#[get("/notifications/product/modal/remove_modal?<id>")]
pub fn remove_modal_product(id: String, cookies: &CookieJar<'_>) {
    let mut all_notification_cookies = Notifications::get_viewed(cookies);

    let current_notification_cookie = all_notification_cookies.iter().position(|x| x.id == id);

    match current_notification_cookie {
        Some(index) => {
            all_notification_cookies[index].time_modal_viewed = Some(chrono::Utc::now());
        }
        None => {
            all_notification_cookies.push(NotificationCookie {
                id: id,
                time_viewed: None,
                time_modal_viewed: Some(chrono::Utc::now()),
            });
        }
    }

    Notifications::update_viewed(&all_notification_cookies, cookies);
}

pub fn routes() -> Vec<Route> {
    routes![
        dashboard,
        remove_banner,
        playground,
        serverless_models_turboframe,
        serverless_pricing_turboframe,
        replace_banner_product,
        remove_modal_product,
        remove_banner_product
    ]
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    Ok(sqlx::migrate!("./migrations").run(pool).await?)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::components::sections::footers::MarketingFooter;
    use crate::guards::Cluster;
    use rocket::fairing::AdHoc;
    use rocket::http::{Cookie, Status};
    use rocket::local::asynchronous::Client;

    #[sqlx::test]
    async fn test_remove_modal() {
        let rocket = rocket::build().mount("/", routes());
        let client = Client::untracked(rocket).await.unwrap();

        let cookie = vec![
            NotificationCookie {
                id: "1".to_string(),
                time_viewed: Some(chrono::Utc::now() - chrono::Duration::days(1)),
                time_modal_viewed: Some(chrono::Utc::now() - chrono::Duration::days(1)),
            },
            NotificationCookie {
                id: "2".to_string(),
                time_viewed: None,
                time_modal_viewed: None,
            },
        ];

        let response = client
            .get("/notifications/product/modal/remove_modal?id=1")
            .private_cookie(Cookie::new("session", Notifications::safe_serialize_session(&cookie)))
            .dispatch()
            .await;

        let time_modal_viewed = Notifications::get_viewed(response.cookies())
            .get(0)
            .unwrap()
            .time_modal_viewed;

        // Update modal view time for existing notification cookie
        assert!(time_modal_viewed.is_some());

        let response = client
            .get("/notifications/product/modal/remove_modal?id=3")
            .private_cookie(Cookie::new("session", Notifications::safe_serialize_session(&cookie)))
            .dispatch()
            .await;

        let time_modal_viewed = Notifications::get_viewed(response.cookies())
            .get(0)
            .unwrap()
            .time_modal_viewed;

        // Update modal view time for new notification cookie
        assert!(time_modal_viewed.is_some());
    }

    #[sqlx::test]
    async fn test_remove_banner_product() {
        let rocket = rocket::build().mount("/", routes());
        let client = Client::untracked(rocket).await.unwrap();

        let cookie = vec![
            NotificationCookie {
                id: "1".to_string(),
                time_viewed: Some(chrono::Utc::now() - chrono::Duration::days(1)),
                time_modal_viewed: Some(chrono::Utc::now() - chrono::Duration::days(1)),
            },
            NotificationCookie {
                id: "2".to_string(),
                time_viewed: None,
                time_modal_viewed: Some(chrono::Utc::now() - chrono::Duration::days(1)),
            },
        ];

        let response = client
            .get("/notifications/product/remove_banner?id=1&target=ajskghjfbs")
            .private_cookie(Cookie::new("session", Notifications::safe_serialize_session(&cookie)))
            .dispatch()
            .await;

        let time_viewed = Notifications::get_viewed(response.cookies())
            .get(0)
            .unwrap()
            .time_viewed;

        // Update view time for existing notification cookie
        assert_eq!(time_viewed.is_some(), true);

        let response = client
            .get("/notifications/product/remove_banner?id=3&target=ajfadghs")
            .private_cookie(Cookie::new("session", Notifications::safe_serialize_session(&cookie)))
            .dispatch()
            .await;

        let time_viewed = Notifications::get_viewed(response.cookies())
            .get(0)
            .unwrap()
            .time_viewed;

        // Update view time for new notification cookie
        assert!(time_viewed.is_some());
    }

    #[sqlx::test]
    async fn test_replace_banner_product() {
        let notification1 = Notification::new("Test notification 1")
            .set_level(&NotificationLevel::ProductMedium)
            .set_deployment("1");
        let notification2 = Notification::new("Test notification 2")
            .set_level(&NotificationLevel::ProductMedium)
            .set_deployment("1");
        let _notification3 = Notification::new("Test notification 3")
            .set_level(&NotificationLevel::ProductMedium)
            .set_deployment("2");
        let _notification4 = Notification::new("Test notification 4").set_level(&NotificationLevel::ProductMedium);
        let _notification5 = Notification::new("Test notification 5").set_level(&NotificationLevel::ProductMarketing);

        let rocket = rocket::build()
            .attach(AdHoc::on_request("request", |req, _| {
                Box::pin(async {
                    req.local_cache(|| Cluster {
                        pool: None,
                        context: Context {
                            user: models::User::default(),
                            cluster: models::Cluster::default(),
                            dropdown_nav: StaticNav { links: vec![] },
                            product_left_nav: StaticNav { links: vec![] },
                            marketing_footer: MarketingFooter::new().render_once().unwrap(),
                            head_items: None,
                        },
                        notifications: Some(vec![
                            Notification::new("Test notification 1")
                                .set_level(&NotificationLevel::ProductMedium)
                                .set_deployment("1"),
                            Notification::new("Test notification 2")
                                .set_level(&NotificationLevel::ProductMedium)
                                .set_deployment("1"),
                            Notification::new("Test notification 3")
                                .set_level(&NotificationLevel::ProductMedium)
                                .set_deployment("2"),
                            Notification::new("Test notification 4").set_level(&NotificationLevel::ProductMedium),
                            Notification::new("Test notification 5").set_level(&NotificationLevel::ProductMarketing),
                        ]),
                    });
                })
            }))
            .mount("/", routes());

        let client = Client::tracked(rocket).await.unwrap();

        let response = client
            .get(format!(
                "/notifications/product/replace_banner?id={}&deployment_id=1",
                notification1.id
            ))
            .dispatch()
            .await;

        let body = response.into_string().await.unwrap();
        let rsp_contains_next_notification = body.contains("Test notification 2");

        // Ensure the banner is replaced with next notification of same type
        assert_eq!(rsp_contains_next_notification, true);

        let response = client
            .get(format!(
                "/notifications/product/replace_banner?id={}&deployment_id=1",
                notification2.id
            ))
            .dispatch()
            .await;

        let body = response.into_string().await.unwrap();
        let rsp_contains_next_notification_3 = body.contains("Test notification 3");
        let rsp_contains_next_notification_4 = body.contains("Test notification 4");
        let rsp_contains_next_notification_5 = body.contains("Test notification 5");

        // Ensure the next notification is not found since none match deployment id or level
        assert_eq!(
            rsp_contains_next_notification_3 && rsp_contains_next_notification_4 && rsp_contains_next_notification_5,
            false
        );
    }

    #[sqlx::test]
    async fn test_replace_banner_product_no_notifications() {
        let notification1 = Notification::new("Test notification 1")
            .set_level(&NotificationLevel::ProductMedium)
            .set_deployment("1");

        let rocket = rocket::build()
            .attach(AdHoc::on_request("request", |req, _| {
                Box::pin(async {
                    req.local_cache(|| Cluster {
                        pool: None,
                        context: Context {
                            user: models::User::default(),
                            cluster: models::Cluster::default(),
                            dropdown_nav: StaticNav { links: vec![] },
                            product_left_nav: StaticNav { links: vec![] },
                            marketing_footer: MarketingFooter::new().render_once().unwrap(),
                            head_items: None,
                        },
                        notifications: None,
                    });
                })
            }))
            .mount("/", routes());

        let client = Client::tracked(rocket).await.unwrap();

        let response = client
            .get(format!(
                "/notifications/product/replace_banner?id={}&deployment_id=1",
                notification1.id
            ))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
    }
}
