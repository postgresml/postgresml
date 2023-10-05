use rocket::route::Route;

pub mod chatbot;
pub mod docs;

pub fn routes() -> Vec<Route> {
    let mut routes = Vec::new();
    routes.extend(docs::routes());
    routes.extend(chatbot::routes());
    routes
}
