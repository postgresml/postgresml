#![allow(renamed_and_removed_lints)]

#[macro_use]
extern crate rocket;

use sqlx::PgPool;

pub mod api;
pub mod catchers;
pub mod components;
pub mod context;
pub mod fairings;
pub mod forms;
pub mod guards;
pub mod models;
pub mod notifications;
pub mod responses;
pub mod routes;
pub mod sentry;
pub mod templates;
pub mod types;
pub mod utils;

#[derive(Debug, Default, Clone)]
pub struct ClustersSettings {
    pub max_connections: u32,
    pub idle_timeout: u64,
    pub min_connections: u32,
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    Ok(sqlx::migrate!("./migrations").run(pool).await?)
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::components::sections::footers::MarketingFooter;
//     use crate::guards::Cluster;
//     use rocket::fairing::AdHoc;
//     use rocket::http::{Cookie, Status};
//     use rocket::local::asynchronous::Client;

//     #[sqlx::test]
//     async fn test_remove_modal() {
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
