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
use crate::utils::cookies::Notifications;
use crate::utils::urls;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
                println!(
                    "{} == {} && {:?} == {:?} && {} == {}",
                    notification.level,
                    desired_level,
                    notification.deployment,
                    deployment_id.clone(),
                    notification.viewed,
                    false
                );
                notification.level == desired_level
                    && notification.deployment == deployment_id
                    && notification.viewed == false
            }
            NotificationLevel::ProductMarketing => notification.level == desired_level && notification.viewed == false,
            _ => false,
        }
    }

    pub fn get_notifications_from_context(context: Option<&crate::guards::Cluster>) -> Option<Notification> {
        match context.as_ref() {
            Some(context) => match &context.notifications {
                Some(notifications) => {
                    return Some(notifications[0].clone());
                }
                None => return None,
            },
            None => return None,
        };
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

    viewed.push(id);
    Notifications::update_viewed(&viewed, cookies);

    let notification = match context.notifications.as_ref() {
        Some(notifications) => {
            if notification_type == "alert" {
                notifications
                    .into_iter()
                    .filter(|n: &&Notification| -> bool { Notification::is_alert(&n.level) && !viewed.contains(&n.id) })
                    .next()
            } else if notification_type == "feature" {
                notifications
                    .into_iter()
                    .filter(|n: &&Notification| -> bool {
                        Notification::is_feature(&n.level) && !viewed.contains(&n.id)
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

// Replace or remove all product banners after user exits out of the message.
#[get("/notifications/product/remove_banner?<id>&<deployment_id>")]
pub fn remove_banner_product(
    id: String,
    deployment_id: Option<String>,
    cookies: &CookieJar<'_>,
    context: &Cluster,
) -> Result<Response, Error> {
    let mut all_viewed = Notifications::get_viewed(cookies);

    all_viewed.push(id.clone());
    Notifications::update_viewed(&all_viewed, cookies);

    // Get the notification that triggered this call.
    // Guaranteed to exist since it built the component that called this, so this is safe to unwrap.
    let last_notification = context
        .notifications
        .as_ref()
        .unwrap()
        .clone()
        .into_iter()
        .filter(|n: &Notification| -> bool { n.id == id })
        .next();

    let next_notification = match context.notifications.as_ref() {
        Some(notifications) => notifications
            .clone()
            .into_iter()
            .filter(|n: &Notification| -> bool {
                let n = n.clone().set_viewed(n.id == id);
                Notification::product_filter(
                    &n,
                    last_notification.clone().unwrap().level.clone(),
                    deployment_id.clone(),
                )
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

pub fn routes() -> Vec<Route> {
    routes![
        dashboard,
        remove_banner,
        playground,
        serverless_models_turboframe,
        serverless_pricing_turboframe,
        remove_banner_product
    ]
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    Ok(sqlx::migrate!("./migrations").run(pool).await?)
}
