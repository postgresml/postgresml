use axum::Router;

use crate::utils::markdown::SiteSearch;

pub mod cms;
pub mod code_editor;
pub mod deployment;

pub fn routes() -> Router<SiteSearch> {
    Router::new().merge(cms::routes()).merge(code_editor::routes())
}
