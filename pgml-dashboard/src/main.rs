use log::{error, info, warn};

use rocket::{catch, catchers, fs::FileServer, get, http::Status, request::Request, response::Redirect};

use pgml_dashboard::{
    guards,
    responses::{self, BadRequest, Response},
    utils::{config, markdown},
};

#[rocket::get("/")]
async fn index() -> Redirect {
    Redirect::to("/dashboard")
}

#[get("/error")]
pub async fn error() -> Result<(), BadRequest> {
    info!("This is additional information for the test");
    error!("This is a test");
    panic!();
}

#[catch(403)]
async fn not_authorized_catcher(_status: Status, _request: &Request<'_>) -> Redirect {
    Redirect::to("/login")
}

#[catch(404)]
async fn not_found_handler(_status: Status, _request: &Request<'_>) -> Response {
    Response::not_found()
}

#[catch(default)]
async fn error_catcher(status: Status, request: &Request<'_>) -> Result<BadRequest, responses::Error> {
    Err(responses::Error(anyhow::anyhow!(
        "{} {}\n{:?}",
        status.code,
        status.reason().unwrap(),
        request
    )))
}

async fn configure_reporting() -> Option<sentry::ClientInitGuard> {
    let mut log_builder = env_logger::Builder::from_default_env();
    log_builder.format_timestamp_micros();

    // TODO move sentry into a once_cell
    let sentry = match config::sentry_dsn() {
        Some(dsn) => {
            // Don't log debug or trace to sentry, regardless of environment
            let logger = log_builder.build();
            let level = logger.filter();
            let logger = sentry_log::SentryLogger::with_dest(logger);
            log::set_boxed_logger(Box::new(logger)).unwrap();
            log::set_max_level(level);

            let name = sentry::release_name!().unwrap_or_else(|| std::borrow::Cow::Borrowed("cloud2"));
            let sha = env!("GIT_SHA");
            let release = format!("{name}+{sha}");
            let result = sentry::init((
                dsn.as_str(),
                sentry::ClientOptions {
                    release: Some(std::borrow::Cow::Owned(release)),
                    debug: true,
                    ..Default::default()
                },
            ));
            info!("Configured reporting w/ Sentry");
            Some(result)
        }
        _ => {
            log_builder.try_init().unwrap();
            info!("Configured reporting w/o Sentry");
            None
        }
    };

    match pgml_dashboard::utils::datadog::client().await {
        Ok(_) => info!("Configured reporting w/ Datadog"),
        Err(err) => warn!("Configured reporting w/o Datadog: {err}"),
    };

    sentry
}

#[rocket::main]
async fn main() {
    #[cfg(tokio_unstable)]
    console_subscriber::init();

    dotenv::dotenv().ok();
    // it's important to hang on to sentry so it isn't dropped and stops reporting
    let _sentry = configure_reporting().await;

    let site_search = markdown::SiteSearch::new()
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

    pgml_dashboard::migrate(guards::Cluster::default().pool())
        .await
        .unwrap();

    let _ = rocket::build()
        .manage(site_search)
        .mount("/", rocket::routes![index, error])
        .mount("/dashboard/static", FileServer::from(config::static_dir()))
        .mount("/dashboard", pgml_dashboard::routes())
        .mount("/engine", pgml_dashboard::api::deployment::routes())
        .mount("/", pgml_dashboard::api::routes())
        .mount("/", rocket::routes![pgml_dashboard::playground])
        .register("/", catchers![error_catcher, not_authorized_catcher, not_found_handler])
        .attach(pgml_dashboard::fairings::RequestMonitor::new())
        .ignite()
        .await
        .expect("failed to ignite Rocket")
        .launch()
        .await
        .expect("failed to shut down Rocket");
}

#[cfg(test)]
mod test {
    use crate::{error, index};
    use pgml_dashboard::guards::Cluster;
    use pgml_dashboard::utils::urls;
    use pgml_dashboard::utils::{config, markdown};
    use rocket::fs::FileServer;
    use rocket::local::asynchronous::Client;
    use rocket::{Build, Rocket};
    use scraper::{Html, Selector};
    use std::vec::Vec;

    async fn rocket() -> Rocket<Build> {
        dotenv::dotenv().ok();

        pgml_dashboard::migrate(Cluster::default().pool()).await.unwrap();

        let mut site_search = markdown::SiteSearch::new()
            .await
            .expect("Error initializing site search");
        site_search.build().await.expect("Error building site search");

        rocket::build()
            .manage(site_search)
            .mount("/", rocket::routes![index, error])
            .mount("/dashboard/static", FileServer::from(config::static_dir()))
            .mount("/dashboard", pgml_dashboard::routes())
            .mount("/engine", pgml_dashboard::api::deployment::routes())
            .mount("/", pgml_dashboard::api::cms::routes())
    }

    fn get_href_links(body: &str, pattern: &str) -> Vec<String> {
        let document = Html::parse_document(body);
        let selector = Selector::parse("a").unwrap();
        let mut output = Vec::<String>::new();
        for element in document.select(&selector) {
            let href = element.value().attr("href").unwrap();
            if href.contains(pattern) && href != pattern {
                output.push(String::from(href));
            }
        }
        output
    }

    #[rocket::async_test]
    async fn test_notebooks_index() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get(urls::deployment_notebooks_turboframe()).dispatch().await;
        assert_eq!(response.status().code, 200);
    }

    #[rocket::async_test]
    async fn test_projects_index() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get(urls::deployment_projects_turboframe()).dispatch().await;
        assert_eq!(response.status().code, 200);
    }

    #[rocket::async_test]
    async fn test_models_index() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get(urls::deployment_models_turboframe()).dispatch().await;
        assert_eq!(response.status().code, 200);
    }

    #[rocket::async_test]
    async fn test_deployments_index() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get("/dashboard/deployments").dispatch().await;
        assert_eq!(response.status().code, 200);
    }

    #[rocket::async_test]
    async fn test_uploader() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get(urls::deployment_uploader_turboframe()).dispatch().await;
        assert_eq!(response.status().code, 200);
    }

    #[rocket::async_test]
    async fn test_snapshots_index() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get(urls::deployment_snapshots_turboframe()).dispatch().await;
        assert_eq!(response.status().code, 200);
    }

    #[rocket::async_test]
    async fn test_snapshot_entries() {
        let snapshots_endpoint = &urls::deployment_snapshots();
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get(snapshots_endpoint).dispatch().await;

        let body = response.into_string().await.unwrap();
        let snapshot_links = get_href_links(body.as_str(), snapshots_endpoint);

        for link in snapshot_links {
            let response = client.get(link.as_str()).dispatch().await;
            assert_eq!(response.status().code, 200);
        }
    }

    #[rocket::async_test]
    async fn test_notebook_entries() {
        let notebooks_endpoint = &urls::deployment_notebooks();
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get(notebooks_endpoint).dispatch().await;

        let body = response.into_string().await.unwrap();
        let notebook_links = get_href_links(body.as_str(), notebooks_endpoint);

        for link in notebook_links {
            let response = client.get(link.as_str()).dispatch().await;
            assert_eq!(response.status().code, 200);
        }
    }

    #[rocket::async_test]
    async fn test_project_entries() {
        let projects_endpoint = &urls::deployment_projects();
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get(projects_endpoint).dispatch().await;

        let body = response.into_string().await.unwrap();
        let project_links = get_href_links(body.as_str(), projects_endpoint);

        for link in project_links {
            let response = client.get(link.as_str()).dispatch().await;
            assert_eq!(response.status().code, 200);
        }
    }

    #[rocket::async_test]
    async fn test_model_entries() {
        let models_endpoint = &urls::deployment_models();
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get(models_endpoint).dispatch().await;

        let body = response.into_string().await.unwrap();
        let model_links = get_href_links(body.as_str(), models_endpoint);

        for link in model_links {
            let response = client.get(link.as_str()).dispatch().await;
            assert_eq!(response.status().code, 200);
        }
    }

    #[rocket::async_test]
    async fn test_deployment_entries() {
        let deployments_endpoint = uri!(crate::api::deployment::deployments_index());
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get(deployments_endpoint).dispatch().await;

        let body = response.into_string().await.unwrap();
        let deployment_links = get_href_links(body.as_str(), deployments_endpoint);

        for link in deployment_links {
            let response = client.get(link.as_str()).dispatch().await;
            assert_eq!(response.status().code, 200);
        }
    }

    #[rocket::async_test]
    async fn test_docs() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get("/docs/").dispatch().await;
        assert_eq!(response.status().code, 200);
    }

    #[rocket::async_test]
    async fn test_blogs() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client
            .get("/blog/postgresml-raises-usd4.7m-to-launch-serverless-ai-application-databases-based-on-postgres")
            .dispatch()
            .await;
        assert_eq!(response.status().code, 200);
    }
}
