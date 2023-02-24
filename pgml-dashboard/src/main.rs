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


mod tests;