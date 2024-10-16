#![allow(renamed_and_removed_lints)]

use axum::extract::Query;
use axum::http::{Request, StatusCode};
use axum::response::Redirect;
use axum::routing::get;
use axum::Extension;
use axum_extra::extract::CookieJar;
use log::{error, info};
use sailfish::TemplateOnce;
use serde::Deserialize;
use sqlx::PgPool;

pub mod api;
pub mod components;
// pub mod fairings;
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
use responses::{BadRequest, Error, Response, ResponseOk};
use templates::{components::StaticNav, *};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use utils::config;
use utils::markdown::SiteSearch;

use crate::components::tables::serverless_models::{ServerlessModels, ServerlessModelsTurbo};
use crate::components::tables::serverless_pricing::{ServerlessPricing, ServerlessPricingTurbo};
use crate::utils::cookies::{NotificationCookie, Notifications};
use crate::utils::urls;
use chrono;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

type Router = axum::Router<SiteSearch>;

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

#[derive(Deserialize)]
struct ServerlessParams {
    style: String,
}

async fn serverless_models_turboframe(Query(ServerlessParams { style }): Query<ServerlessParams>) -> ResponseOk {
    let comp = ServerlessModels::new().set_style_type(&style);
    ResponseOk(ServerlessModelsTurbo::new(comp.into()).render_once().unwrap())
}

async fn serverless_pricing_turboframe(Query(ServerlessParams { style }): Query<ServerlessParams>) -> ResponseOk {
    let comp = ServerlessPricing::new().set_style_type(&style);
    ResponseOk(ServerlessPricingTurbo::new(comp.into()).render_once().unwrap())
}

#[derive(Deserialize)]
struct DashboardParams {
    tab: Option<String>,
    id: Option<i64>,
}

// Reroute old style query style dashboard links.
async fn dashboard(Query(DashboardParams { tab, id }): Query<DashboardParams>) -> Redirect {
    let tab = tab.unwrap_or("Notebooks".into());

    match tab.as_str() {
        "Notebooks" => Redirect::to(&urls::deployment_notebooks()),

        "Notebook" => match id {
            Some(id) => Redirect::to(&urls::deployment_notebook_by_id(id)),
            None => Redirect::to(&urls::deployment_notebooks()),
        },

        "Projects" => Redirect::to(&urls::deployment_projects()),

        "Project" => match id {
            Some(id) => Redirect::to(&urls::deployment_project_by_id(id)),
            None => Redirect::to(&urls::deployment_projects()),
        },

        "Models" => Redirect::to(&urls::deployment_models()),

        "Model" => match id {
            Some(id) => Redirect::to(&urls::deployment_model_by_id(id)),
            None => Redirect::to(&urls::deployment_models()),
        },

        "Snapshots" => Redirect::to(&urls::deployment_snapshots()),

        "Snapshot" => match id {
            Some(id) => Redirect::to(&urls::deployment_snapshot_by_id(id)),
            None => Redirect::to(&urls::deployment_snapshots()),
        },

        "Upload_Data" => Redirect::to(&urls::deployment_uploader()),
        _ => Redirect::to(&urls::deployment_notebooks()),
    }
}

pub async fn playground(Extension(cluster): Extension<Cluster>) -> Result<ResponseOk, Error> {
    let mut layout = crate::templates::WebAppBase::new("Playground", &cluster);
    Ok(ResponseOk(layout.render(templates::Playground {})))
}

#[derive(Deserialize)]
struct RemoveBannerParams {
    id: String,
    notification_type: String,
}

// Remove Alert and Feature banners after user exits out of the message.
async fn remove_banner(
    Query(RemoveBannerParams { id, notification_type }): Query<RemoveBannerParams>,
    cookies: CookieJar,
    Extension(context): Extension<Cluster>,
) -> ResponseOk {
    let mut viewed = Notifications::get_viewed(&cookies);

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

#[derive(Deserialize)]
struct ReplaceBannerParams {
    id: String,
    deployment_id: Option<String>,
}

// Replace a product banner after user exits out of the message.
async fn replace_banner_product(
    Query(ReplaceBannerParams { id, deployment_id }): Query<ReplaceBannerParams>,
    cookies: CookieJar,
    Extension(context): Extension<Cluster>,
) -> Result<Response, Error> {
    let mut all_notification_cookies = Notifications::get_viewed(&cookies);
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

#[derive(Deserialize)]
struct RemoveBannerProductParams {
    id: String,
    target: String,
}

// Remove a product banners after user exits out of the message.
async fn remove_banner_product(
    Query(RemoveBannerProductParams { id, target }): Query<RemoveBannerProductParams>,
    cookies: CookieJar,
) -> Result<Response, Error> {
    let mut all_notification_cookies = Notifications::get_viewed(&cookies);

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

#[derive(Deserialize)]
struct RemoveModalParams {
    id: String,
}

// Update cookie to show the user has viewed the modal.
async fn remove_modal_product(Query(RemoveModalParams { id }): Query<RemoveModalParams>, cookies: CookieJar) {
    let mut all_notification_cookies = Notifications::get_viewed(&cookies);

    let current_notification_cookie = all_notification_cookies.iter().position(|x| x.id == id);

    match current_notification_cookie {
        Some(index) => {
            all_notification_cookies[index].time_modal_viewed = Some(chrono::Utc::now());
        }
        None => {
            all_notification_cookies.push(NotificationCookie {
                id,
                time_viewed: None,
                time_modal_viewed: Some(chrono::Utc::now()),
            });
        }
    }

    Notifications::update_viewed(&all_notification_cookies, cookies);
}

pub fn routes() -> Router {
    axum::Router::new()
        .route("/dashboard", get(dashboard))
        .route("/notifications/remove_banner", get(remove_banner))
        .route("/playground", get(playground))
        .route("/serverless_models/turboframe", get(serverless_models_turboframe))
        .route("/serverless_pricing/turboframe", get(serverless_pricing_turboframe))
        .route("/error", get(error))
        .route("/notifications/product/replace_banner", get(replace_banner_product))
        .route("/notifications/product/modal/remove_modal", get(remove_modal_product))
        .route("/notifications/product/remove_banner", get(remove_banner_product))
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    Ok(sqlx::migrate!("./migrations").run(pool).await?)
}

pub async fn error() -> Result<(), BadRequest> {
    info!("This is additional information for the test");
    error!("This is a test");
    panic!();
}

// #[catch(403)]
pub async fn not_authorized_catcher() -> Redirect {
    Redirect::to("/login")
}

// #[catch(404)]
pub async fn not_found_handler() -> Response {
    Response::not_found()
}

// #[catch(default)]
pub async fn error_catcher(status: StatusCode, request: Request<()>) -> Result<BadRequest, responses::Error> {
    Err(responses::Error(anyhow::anyhow!("{}\n{:?}", status, request)))
}

pub async fn app() -> axum::Router {
    let site_search = utils::markdown::SiteSearch::new()
        .await
        .expect("Error initializing site search");
    let mut site_search_copy = site_search.clone();
    tokio::spawn(async move {
        match site_search_copy.build().await {
            Err(e) => {
                error!("Error building site search: {e}")
            }
            _ => {}
        };
    });

    Router::new()
        .route("/", get(|| async { Redirect::permanent("/dashboard") }))
        .nest("/dashboard", routes())
        .nest("/engine", api::deployment::routes())
        .nest("/", api::routes())
        .layer(Extension(Cluster::default()))
        .layer(TraceLayer::new_for_http())
        .nest_service("/dashboard/static", ServeDir::new(config::static_dir()))
        .fallback(not_found_handler)
        .with_state(site_search)
}

// TODO: Fix tests
// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::components::sections::footers::MarketingFooter;
//     use crate::guards::Cluster;

//     #[sqlx::test]
//     async fn test_remove_modal() {
//         // let rocket = rocket::build().mount("/", routes());
//         let client = Client::untracked(rocket).await.unwrap();

//         let cookie = vec![
//             NotificationCookie {
//                 id: "1".to_string(),
//                 time_viewed: Some(chrono::Utc::now() - chrono::Duration::days(1)),
//                 time_modal_viewed: Some(chrono::Utc::now() - chrono::Duration::days(1)),
//             },
//             NotificationCookie {
//                 id: "2".to_string(),
//                 time_viewed: None,
//                 time_modal_viewed: None,
//             },
//         ];

//         let response = client
//             .get("/notifications/product/modal/remove_modal?id=1")
//             .private_cookie(Cookie::new("session", Notifications::safe_serialize_session(&cookie)))
//             .dispatch()
//             .await;

//         let time_modal_viewed = Notifications::get_viewed(response.cookies())
//             .get(0)
//             .unwrap()
//             .time_modal_viewed;

//         // Update modal view time for existing notification cookie
//         assert!(time_modal_viewed.is_some());

//         let response = client
//             .get("/notifications/product/modal/remove_modal?id=3")
//             .private_cookie(Cookie::new("session", Notifications::safe_serialize_session(&cookie)))
//             .dispatch()
//             .await;

//         let time_modal_viewed = Notifications::get_viewed(response.cookies())
//             .get(0)
//             .unwrap()
//             .time_modal_viewed;

//         // Update modal view time for new notification cookie
//         assert!(time_modal_viewed.is_some());
//     }

//     #[sqlx::test]
//     async fn test_remove_banner_product() {
//         let rocket = rocket::build().mount("/", routes());
//         let client = Client::untracked(rocket).await.unwrap();

//         let cookie = vec![
//             NotificationCookie {
//                 id: "1".to_string(),
//                 time_viewed: Some(chrono::Utc::now() - chrono::Duration::days(1)),
//                 time_modal_viewed: Some(chrono::Utc::now() - chrono::Duration::days(1)),
//             },
//             NotificationCookie {
//                 id: "2".to_string(),
//                 time_viewed: None,
//                 time_modal_viewed: Some(chrono::Utc::now() - chrono::Duration::days(1)),
//             },
//         ];

//         let response = client
//             .get("/notifications/product/remove_banner?id=1&target=ajskghjfbs")
//             .private_cookie(Cookie::new("session", Notifications::safe_serialize_session(&cookie)))
//             .dispatch()
//             .await;

//         let time_viewed = Notifications::get_viewed(response.cookies())
//             .get(0)
//             .unwrap()
//             .time_viewed;

//         // Update view time for existing notification cookie
//         assert_eq!(time_viewed.is_some(), true);

//         let response = client
//             .get("/notifications/product/remove_banner?id=3&target=ajfadghs")
//             .private_cookie(Cookie::new("session", Notifications::safe_serialize_session(&cookie)))
//             .dispatch()
//             .await;

//         let time_viewed = Notifications::get_viewed(response.cookies())
//             .get(0)
//             .unwrap()
//             .time_viewed;

//         // Update view time for new notification cookie
//         assert!(time_viewed.is_some());
//     }

//     #[sqlx::test]
//     async fn test_replace_banner_product() {
//         let notification1 = Notification::new("Test notification 1")
//             .set_level(&NotificationLevel::ProductMedium)
//             .set_deployment("1");
//         let notification2 = Notification::new("Test notification 2")
//             .set_level(&NotificationLevel::ProductMedium)
//             .set_deployment("1");
//         let _notification3 = Notification::new("Test notification 3")
//             .set_level(&NotificationLevel::ProductMedium)
//             .set_deployment("2");
//         let _notification4 = Notification::new("Test notification 4").set_level(&NotificationLevel::ProductMedium);
//         let _notification5 = Notification::new("Test notification 5").set_level(&NotificationLevel::ProductMarketing);

//         let rocket = rocket::build()
//             .attach(AdHoc::on_request("request", |req, _| {
//                 Box::pin(async {
//                     req.local_cache(|| Cluster {
//                         pool: None,
//                         context: Context {
//                             user: models::User::default(),
//                             cluster: models::Cluster::default(),
//                             dropdown_nav: StaticNav { links: vec![] },
//                             product_left_nav: StaticNav { links: vec![] },
//                             marketing_footer: MarketingFooter::new().render_once().unwrap(),
//                             head_items: None,
//                         },
//                         notifications: Some(vec![
//                             Notification::new("Test notification 1")
//                                 .set_level(&NotificationLevel::ProductMedium)
//                                 .set_deployment("1"),
//                             Notification::new("Test notification 2")
//                                 .set_level(&NotificationLevel::ProductMedium)
//                                 .set_deployment("1"),
//                             Notification::new("Test notification 3")
//                                 .set_level(&NotificationLevel::ProductMedium)
//                                 .set_deployment("2"),
//                             Notification::new("Test notification 4").set_level(&NotificationLevel::ProductMedium),
//                             Notification::new("Test notification 5").set_level(&NotificationLevel::ProductMarketing),
//                         ]),
//                     });
//                 })
//             }))
//             .mount("/", routes());

//         let client = Client::tracked(rocket).await.unwrap();

//         let response = client
//             .get(format!(
//                 "/notifications/product/replace_banner?id={}&deployment_id=1",
//                 notification1.id
//             ))
//             .dispatch()
//             .await;

//         let body = response.into_string().await.unwrap();
//         let rsp_contains_next_notification = body.contains("Test notification 2");

//         // Ensure the banner is replaced with next notification of same type
//         assert_eq!(rsp_contains_next_notification, true);

//         let response = client
//             .get(format!(
//                 "/notifications/product/replace_banner?id={}&deployment_id=1",
//                 notification2.id
//             ))
//             .dispatch()
//             .await;

//         let body = response.into_string().await.unwrap();
//         let rsp_contains_next_notification_3 = body.contains("Test notification 3");
//         let rsp_contains_next_notification_4 = body.contains("Test notification 4");
//         let rsp_contains_next_notification_5 = body.contains("Test notification 5");

//         // Ensure the next notification is not found since none match deployment id or level
//         assert_eq!(
//             rsp_contains_next_notification_3 && rsp_contains_next_notification_4 && rsp_contains_next_notification_5,
//             false
//         );
//     }

//     #[sqlx::test]
//     async fn test_replace_banner_product_no_notifications() {
//         let notification1 = Notification::new("Test notification 1")
//             .set_level(&NotificationLevel::ProductMedium)
//             .set_deployment("1");

//         let rocket = rocket::build()
//             .attach(AdHoc::on_request("request", |req, _| {
//                 Box::pin(async {
//                     req.local_cache(|| Cluster {
//                         pool: None,
//                         context: Context {
//                             user: models::User::default(),
//                             cluster: models::Cluster::default(),
//                             dropdown_nav: StaticNav { links: vec![] },
//                             product_left_nav: StaticNav { links: vec![] },
//                             marketing_footer: MarketingFooter::new().render_once().unwrap(),
//                             head_items: None,
//                         },
//                         notifications: None,
//                     });
//                 })
//             }))
//             .mount("/", routes());

//         let client = Client::tracked(rocket).await.unwrap();

//         let response = client
//             .get(format!(
//                 "/notifications/product/replace_banner?id={}&deployment_id=1",
//                 notification1.id
//             ))
//             .dispatch()
//             .await;

//         assert_eq!(response.status(), Status::Ok);
//     }
// }
