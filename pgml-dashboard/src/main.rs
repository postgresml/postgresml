use log::{error, info, warn};
use rocket::{
    catch, catchers,
    fs::FileServer,
    get,
    http::Status,
    request::Request,
    response::Redirect,
};

use pgml_dashboard::{responses::{self, BadRequest, Response}, utils::{config, markdown}};

#[rocket::get("/")]
async fn index() -> Redirect {
    Redirect::to("/dashboard/notebooks")
}

#[get("/error")]
pub async fn error() -> Result<(), BadRequest> {
    info!("This is additional information for the test");
    error!("This is a test");
    let error: Option<i32> = None;
    error.unwrap();
    Ok(())
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
async fn error_catcher(status: Status, request: &Request<'_>) -> Result<BadRequest, responses::Error>  {
    Err(responses::Error(anyhow::anyhow!("{} {}\n{:?}", status.code, status.reason().unwrap(), request)))
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

            let name =
                sentry::release_name!().unwrap_or_else(|| std::borrow::Cow::Borrowed("cloud2"));
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
    dotenv::dotenv().ok();
    // it's important to hang on to sentry so it isn't dropped and stops reporting
    let _sentry = configure_reporting().await;

    if config::dev_mode() {
        warn!("============================================");
        warn!("PostgresML is set to run in development mode");
        warn!("============================================");

        let status = tokio::process::Command::new("npm")
            .arg("exec")
            .arg("sass")
            .arg("static/css/bootstrap-theme.scss")
            .arg("static/css/style.css")
            .status()
            .await
            .unwrap();

        if !status.success() {
            error!("SCSS compilation failed. Do you have `node`, `npm`, and `sass` installed and working globally?");
            std::process::exit(1);
        }
    }

    markdown::SearchIndex::build().await.unwrap();

    let clusters = pgml_dashboard::Clusters::new();
    let settings = pgml_dashboard::ClustersSettings {
        min_connections: 0,
        max_connections: 5,
        idle_timeout: 15_000,
    };

    clusters
        .add(
            -1,
            &pgml_dashboard::guards::default_database_url(),
            settings,
        )
        .unwrap();

    pgml_dashboard::migrate(&clusters.get(-1).unwrap())
        .await
        .unwrap();

    let _ = rocket::build()
        .manage(clusters)
        .manage(markdown::SearchIndex::open().unwrap())
        .mount("/", rocket::routes![index, error])
        .mount("/dashboard/static", FileServer::from(&config::static_dir()))
        .mount("/dashboard", pgml_dashboard::routes())
        .mount("/", pgml_dashboard::api::docs::routes())
        .register(
            "/",
            catchers![error_catcher, not_authorized_catcher, not_found_handler],
        )
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
    use pgml_dashboard::Clusters;
    use pgml_dashboard::{index, migrate, paths};
    use rocket::fs::FileServer;
    use rocket::local::asynchronous::Client;
    use rocket::{Build, Rocket};
    use scraper::{Html, Selector};
    use std::vec::Vec;

    async fn rocket() -> Rocket<Build> {
        let max_connections = 5;
        let min_connections = 1;
        let idle_timeout = 15_000;

        let clusters = Clusters::new();
        clusters
            .add(-1, &pgml_dashboard::guards::default_database_url())
            .unwrap();

        migrate(&clusters.get(-1).unwrap()).await.unwrap();

        rocket::build()
            .manage(clusters)
            .mount("/dashboard", paths())
            .mount("/static", FileServer::from("static"))
            .mount("/docs", pgml_dashboard::api::docs::routes())
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
        let response = client.get("/dashboard/notebooks/").dispatch().await;
        assert_eq!(response.status().code, 200);
    }

    #[rocket::async_test]
    async fn test_projects_index() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get("/dashboard/projects/").dispatch().await;
        assert_eq!(response.status().code, 200);
    }

    #[rocket::async_test]
    async fn test_models_index() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get("/dashboard/models/").dispatch().await;
        assert_eq!(response.status().code, 200);
    }

    #[rocket::async_test]
    async fn test_deployments_index() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get("/dashboard/deployments/").dispatch().await;
        assert_eq!(response.status().code, 200);
    }

    #[rocket::async_test]
    async fn test_uploader() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get("/dashboard/uploader/").dispatch().await;
        assert_eq!(response.status().code, 200);
    }

    #[rocket::async_test]
    async fn test_snapshots_index() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get("/dashboard/snapshots/").dispatch().await;
        assert_eq!(response.status().code, 200);
    }

    #[rocket::async_test]
    async fn test_snapshot_entries() {
        let snapshots_endpoint = "/dashboard/snapshots/";
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
        let notebooks_endpoint = "/dashboard/notebooks/";
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
        let projects_endpoint = "/dashboard/projects/";
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
        let models_endpoint = "/dashboard/models/";
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
        let deployments_endpoint = "/deployments/";
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get(deployments_endpoint).dispatch().await;

        let body = response.into_string().await.unwrap();
        let deployment_links = get_href_links(body.as_str(), deployments_endpoint);

        for link in deployment_links {
            let response = client.get(link.as_str()).dispatch().await;
            assert_eq!(response.status().code, 200);
        }
    }
}
