use rocket::{Rocket, Build, Orbit};
use rocket::fs::FileServer;
use pgml_dashboard::{Clusters};
use pgml_dashboard::{migrate,paths,index};

async fn rocket() -> Rocket<Build> {
    let clusters = Clusters::new();
    clusters
        .add(-1, &pgml_dashboard::guards::default_database_url())
        .unwrap();

        migrate(&clusters.get(-1).unwrap())
        .await
        .unwrap();
   
   rocket::build()
        .manage(clusters)
        .mount("/", rocket::routes![index,])
        .mount("/dashboard/static", FileServer::from("static"))
        .mount("/dashboard", paths())

}

#[cfg(test)]
mod test {
    use super::rocket;
    use rocket::local::asynchronous::Client;
    
    #[rocket::async_test]
    async fn test_notebooks_index() {
        let client = Client::tracked(rocket().await).await.unwrap();
        let response = client.get("/dashboard/notebooks/").dispatch().await;
        assert_eq!(response.status().code,200);
    }
}