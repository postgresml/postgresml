use rocket::{http::CookieJar, response::Redirect, Route};
use sailfish::TemplateOnce;

use crate::{
    components::{
        notifications::{
            marketing::{AlertBanner, FeatureBanner},
            product::ProductBanner,
        },
        tables::{
            serverless_models::ServerlessModelsTurbo, serverless_pricing::ServerlessPricingTurbo, ServerlessModels,
            ServerlessPricing,
        },
    },
    guards::Cluster,
    notifications::Notification,
    responses::{BadRequest, Error, Response, ResponseOk},
    templates,
    utils::{
        cookies::{NotificationCookie, Notifications},
        urls,
    },
};

#[get("/")]
pub async fn index() -> Redirect {
    Redirect::to("/dashboard")
}

#[get("/error")]
pub async fn error() -> Result<(), BadRequest> {
    info!("This is additional information for the test");
    error!("This is a test");
    panic!();
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
                id,
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
