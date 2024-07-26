use crate::app::App;
use axum::response::Response as AxumResponse;
use axum::{
    body::Body,
    extract::State,
    http::{Request, Response, StatusCode},
    response::IntoResponse,
};
use leptos::*;
use tower::ServiceExt;
use tower_http::services::ServeDir;

pub async fn file_and_error_handler(State(options): State<LeptosOptions>, req: Request<Body>) -> AxumResponse {
    let root = options.site_root.clone();
    let (parts, body) = req.into_parts();

    let mut static_parts = parts.clone();
    static_parts.headers.clear();
    if let Some(encodings) = parts.headers.get("accept-encoding") {
        static_parts.headers.insert("accept-encoding", encodings.clone());
    }

    let res = get_static_file(Request::from_parts(static_parts, Body::empty()), &root)
        .await
        .unwrap();

    if res.status() == StatusCode::OK {
        res.into_response()
    } else {
        let handler = leptos_axum::render_app_to_stream(options.to_owned(), App);
        handler(Request::from_parts(parts, body)).await.into_response()
    }
}

async fn get_static_file(request: Request<Body>, root: &str) -> Result<Response<Body>, (StatusCode, String)> {
    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    // This path is relative to the cargo root
    match ServeDir::new(root)
        .precompressed_gzip()
        .precompressed_br()
        .oneshot(request)
        .await
    {
        Ok(res) => Ok(res.into_response()),
        Err(err) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Error serving files: {err}"))),
    }
}
