#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use pgml_dashboard::app::*;
    use pgml_dashboard::fileserv::file_and_error_handler;

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // build our application with a route
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, App)
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{addr}");
    axum::serve(listener, app.into_make_service()).await.unwrap();

    // #[cfg(tokio_unstable)]
    //  console_subscriber::init();

    //  dotenv::dotenv().ok();
    //  // it's important to hang on to sentry so it isn't dropped and stops reporting
    //  let _sentry = configure_reporting().await;

    //  let site_search = markdown::SiteSearch::new()
    //      .await
    //      .expect("Error initializing site search");
    //  let mut site_search_copy = site_search.clone();
    //  tokio::spawn(async move {
    //      if let Err(e) = site_search_copy.build().await {
    //          error!("Error building site search: {e}")
    //      }
    //  });

    //  pgml_dashboard::migrate(guards::Cluster::default().pool())
    //      .await
    //      .unwrap();

    //  let _ = rocket::build()
    //      .manage(site_search)
    //      .mount("/", rocket::routes![index, error])
    //      .mount("/dashboard/static", FileServer::from(config::static_dir()))
    //      .mount("/dashboard", pgml_dashboard::routes::routes())
    //      .mount("/engine", pgml_dashboard::api::deployment::routes())
    //      .mount("/", pgml_dashboard::api::routes())
    //      .mount("/", rocket::routes![pgml_dashboard::routes::playground])
    //      .register("/", catchers![error_catcher, not_authorized_catcher, not_found_handler])
    //      .attach(pgml_dashboard::fairings::RequestMonitor::new())
    //      .ignite()
    //      .await
    //      .expect("failed to ignite Rocket")
    //      .launch()
    //      .await
    //      .expect("failed to shut down Rocket");
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}

// #[cfg(test)]
// mod test {
//     use crate::{error, index};
//     use pgml_dashboard::guards::Cluster;
//     use pgml_dashboard::utils::urls;
//     use pgml_dashboard::utils::{config, markdown};
//     use rocket::fs::FileServer;
//     use rocket::local::asynchronous::Client;
//     use rocket::{Build, Rocket};
//     use scraper::{Html, Selector};
//     use std::vec::Vec;

//     async fn rocket() -> Rocket<Build> {
//         dotenv::dotenv().ok();

//         pgml_dashboard::migrate(Cluster::default().pool()).await.unwrap();

//         let mut site_search = markdown::SiteSearch::new()
//             .await
//             .expect("Error initializing site search");
//         site_search.build().await.expect("Error building site search");

//         rocket::build()
//             .manage(site_search)
//             .mount("/", rocket::routes![index, error])
//             .mount("/dashboard/static", FileServer::from(config::static_dir()))
//             .mount("/dashboard", pgml_dashboard::routes())
//             .mount("/engine", pgml_dashboard::api::deployment::routes())
//             .mount("/", pgml_dashboard::api::cms::routes())
//     }

//     fn get_href_links(body: &str, pattern: &str) -> Vec<String> {
//         let document = Html::parse_document(body);
//         let selector = Selector::parse("a").unwrap();
//         let mut output = Vec::<String>::new();
//         for element in document.select(&selector) {
//             let href = element.value().attr("href").unwrap();
//             if href.contains(pattern) && href != pattern {
//                 output.push(String::from(href));
//             }
//         }
//         output
//     }

//     #[rocket::async_test]
//     async fn test_notebooks_index() {
//         let client = Client::tracked(rocket().await).await.unwrap();
//         let response = client.get(urls::deployment_notebooks_turboframe()).dispatch().await;
//         assert_eq!(response.status().code, 200);
//     }

//     #[rocket::async_test]
//     async fn test_projects_index() {
//         let client = Client::tracked(rocket().await).await.unwrap();
//         let response = client.get(urls::deployment_projects_turboframe()).dispatch().await;
//         assert_eq!(response.status().code, 200);
//     }

//     #[rocket::async_test]
//     async fn test_models_index() {
//         let client = Client::tracked(rocket().await).await.unwrap();
//         let response = client.get(urls::deployment_models_turboframe()).dispatch().await;
//         assert_eq!(response.status().code, 200);
//     }

//     #[rocket::async_test]
//     async fn test_deployments_index() {
//         let client = Client::tracked(rocket().await).await.unwrap();
//         let response = client.get("/dashboard/deployments").dispatch().await;
//         assert_eq!(response.status().code, 200);
//     }

//     #[rocket::async_test]
//     async fn test_uploader() {
//         let client = Client::tracked(rocket().await).await.unwrap();
//         let response = client.get(urls::deployment_uploader_turboframe()).dispatch().await;
//         assert_eq!(response.status().code, 200);
//     }

//     #[rocket::async_test]
//     async fn test_snapshots_index() {
//         let client = Client::tracked(rocket().await).await.unwrap();
//         let response = client.get(urls::deployment_snapshots_turboframe()).dispatch().await;
//         assert_eq!(response.status().code, 200);
//     }

//     #[rocket::async_test]
//     async fn test_snapshot_entries() {
//         let snapshots_endpoint = &urls::deployment_snapshots();
//         let client = Client::tracked(rocket().await).await.unwrap();
//         let response = client.get(snapshots_endpoint).dispatch().await;

//         let body = response.into_string().await.unwrap();
//         let snapshot_links = get_href_links(body.as_str(), snapshots_endpoint);

//         for link in snapshot_links {
//             let response = client.get(link.as_str()).dispatch().await;
//             assert_eq!(response.status().code, 200);
//         }
//     }

//     #[rocket::async_test]
//     async fn test_notebook_entries() {
//         let notebooks_endpoint = &urls::deployment_notebooks();
//         let client = Client::tracked(rocket().await).await.unwrap();
//         let response = client.get(notebooks_endpoint).dispatch().await;

//         let body = response.into_string().await.unwrap();
//         let notebook_links = get_href_links(body.as_str(), notebooks_endpoint);

//         for link in notebook_links {
//             let response = client.get(link.as_str()).dispatch().await;
//             assert_eq!(response.status().code, 200);
//         }
//     }

//     #[rocket::async_test]
//     async fn test_project_entries() {
//         let projects_endpoint = &urls::deployment_projects();
//         let client = Client::tracked(rocket().await).await.unwrap();
//         let response = client.get(projects_endpoint).dispatch().await;

//         let body = response.into_string().await.unwrap();
//         let project_links = get_href_links(body.as_str(), projects_endpoint);

//         for link in project_links {
//             let response = client.get(link.as_str()).dispatch().await;
//             assert_eq!(response.status().code, 200);
//         }
//     }

//     #[rocket::async_test]
//     async fn test_model_entries() {
//         let models_endpoint = &urls::deployment_models();
//         let client = Client::tracked(rocket().await).await.unwrap();
//         let response = client.get(models_endpoint).dispatch().await;

//         let body = response.into_string().await.unwrap();
//         let model_links = get_href_links(body.as_str(), models_endpoint);

//         for link in model_links {
//             let response = client.get(link.as_str()).dispatch().await;
//             assert_eq!(response.status().code, 200);
//         }
//     }

//     #[rocket::async_test]
//     async fn test_deployment_entries() {
//         let deployments_endpoint = "/deployments";
//         let client = Client::tracked(rocket().await).await.unwrap();
//         let response = client.get(deployments_endpoint).dispatch().await;

//         let body = response.into_string().await.unwrap();
//         let deployment_links = get_href_links(body.as_str(), deployments_endpoint);

//         for link in deployment_links {
//             let response = client.get(link.as_str()).dispatch().await;
//             assert_eq!(response.status().code, 200);
//         }
//     }

//     #[rocket::async_test]
//     async fn test_docs() {
//         let client = Client::tracked(rocket().await).await.unwrap();
//         let response = client.get("/docs/").dispatch().await;
//         assert_eq!(response.status().code, 200);
//     }

//     #[rocket::async_test]
//     async fn test_blogs() {
//         let client = Client::tracked(rocket().await).await.unwrap();
//         let response = client
//             .get("/blog/postgresml-raises-usd4.7m-to-launch-serverless-ai-application-databases-based-on-postgres")
//             .dispatch()
//             .await;
//         assert_eq!(response.status().code, 200);
//     }
// }
