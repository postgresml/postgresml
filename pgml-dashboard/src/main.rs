use rocket::fs::FileServer;
use rocket::response::Redirect;

#[rocket::get("/")]
async fn index() -> Redirect {
    Redirect::to("/dashboard")
}

#[rocket::main]
async fn main() {
    let clusters = pgml_dashboard::Clusters::new();
    clusters
        .add(-1, &pgml_dashboard::guards::default_database_url())
        .unwrap();

    pgml_dashboard::migrate(&clusters.get(-1).unwrap())
        .await
        .unwrap();

    let _ = rocket::build()
        .manage(clusters)
        .mount("/", rocket::routes![index,])
        .mount("/dashboard/static", FileServer::from("static"))
        .mount("/dashboard", pgml_dashboard::paths())
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
        let clusters = Clusters::new();
        clusters
            .add(-1, &pgml_dashboard::guards::default_database_url())
            .unwrap();

        migrate(&clusters.get(-1).unwrap()).await.unwrap();

        rocket::build()
            .manage(clusters)
            .mount("/", rocket::routes![index,])
            .mount("/dashboard/static", FileServer::from("static"))
            .mount("/dashboard", paths())
    }

    fn get_href_links(body: &str, pattern: &str) -> Vec<String> {
        let document = Html::parse_document(body);
        let selector = Selector::parse("a").unwrap();
        let mut output = Vec::<String>::new();
        for element in document.select(&selector) {
            let href =  element.value().attr("href").unwrap();
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
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get("/dashboard/snapshots/").dispatch().await;

        let body = response.into_string().await.unwrap();
        let snapshot_links =  get_href_links(body.as_str(),"/dashboard/snapshots/");

        for link in snapshot_links {
            let response = client.get(link.as_str()).dispatch().await;
            assert_eq!(response.status().code, 200);
        }
        
        
    }
}
