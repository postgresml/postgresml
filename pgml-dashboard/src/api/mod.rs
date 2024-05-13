use rocket::route::Route;

pub mod chatbot;
pub mod cms;

pub fn routes() -> Vec<Route> {
    let mut routes = Vec::new();
    routes.extend(cms::routes());
    routes.extend(chatbot::routes());
    routes
}
