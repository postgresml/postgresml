use axum::Router;

pub mod cms;
pub mod code_editor;
pub mod deployment;

pub fn routes() -> Router {
    Router::new().merge(cms::routes()).merge(code_editor::routes())
}
