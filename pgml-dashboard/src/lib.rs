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
use guards::Cluster;
use responses::{Error, ResponseOk};
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
        }
    }

    pub fn level(mut self, level: &NotificationLevel) -> Notification {
        self.level = level.clone();
        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Notification {
        self.dismissible = dismissible;
        self
    }

    pub fn link(mut self, link: &str) -> Notification {
        self.link = Some(link.into());
        self
    }

    pub fn viewed(mut self, viewed: bool) -> Notification {
        self.viewed = viewed;
        self
    }

    pub fn is_alert(level: &NotificationLevel) -> bool {
        match level {
            NotificationLevel::Level1 => true,
            NotificationLevel::Level2 => true,
            NotificationLevel::Level3 => true,
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
                        .filter(|n| !Notification::is_alert(&n.level))
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
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum NotificationLevel {
    #[default]
    Level1,
    Level2,
    Level3,
    Feature1,
    Feature2,
    Feature3,
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
    let mut layout = crate::templates::WebAppBase::new("Playground", &cluster.context);
    Ok(ResponseOk(layout.render(templates::Playground {})))
}

#[get("/notifications/remove_banner?<id>&<alert>")]
pub fn remove_banner(id: String, alert: bool, cookies: &CookieJar<'_>, context: &Cluster) -> ResponseOk {
    let mut viewed = Notifications::get_viewed(cookies);

    viewed.push(id);
    Notifications::update_viewed(&viewed, cookies);

    let notification = match context.notifications.as_ref() {
        Some(notifications) => {
            if alert {
                notifications
                    .into_iter()
                    .filter(|n: &&Notification| -> bool { Notification::is_alert(&n.level) && !viewed.contains(&n.id) })
                    .next()
            } else {
                notifications
                    .into_iter()
                    .filter(|n: &&Notification| -> bool {
                        !Notification::is_alert(&n.level) && !viewed.contains(&n.id)
                    })
                    .next()
            }
        }
        _ => None,
    };

    if alert {
        return ResponseOk(AlertBanner::from_notification(notification).render_once().unwrap());
    } else {
        return ResponseOk(FeatureBanner::from_notification(notification).render_once().unwrap());
    }
}

pub fn routes() -> Vec<Route> {
    routes![
        dashboard,
        remove_banner,
        playground,
        serverless_models_turboframe,
        serverless_pricing_turboframe
    ]
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    Ok(sqlx::migrate!("./migrations").run(pool).await?)
}
