use axum::{extract::Query, routing::get, Router};
use sailfish::TemplateOnce;
use serde::Deserialize;

use crate::{
    guards::Cluster,
    guards::ConnectedCluster,
    responses::{Error, ResponseOk},
};

use crate::templates::{components::NavLink, *};

use crate::templates;
use crate::utils::tabs;
use crate::utils::urls;

pub fn routes() -> Router {
    Router::new()
        .route("/uploader", get(uploader))
        .route("/uploader_turboframe", get(uploader_index))
        .route("/uploader_turboframe/done", get(uploaded_index))
}

// Returns the uploader page.
pub async fn uploader(cluster: &Cluster, _connected: ConnectedCluster) -> Result<ResponseOk, Error> {
    let mut layout = crate::templates::WebAppBase::new("Dashboard", &cluster);
    layout.breadcrumbs(vec![NavLink::new("Upload Data", &urls::deployment_uploader()).active()]);

    let tabs = vec![tabs::Tab {
        name: "Upload data",
        content: UploaderTab { table_name: None }.render_once().unwrap(),
    }];

    let nav_tabs = tabs::Tabs::new(tabs, Some("Upload Data"), Some("Upload Data"))?;

    Ok(ResponseOk(layout.render(templates::Dashboard::new(nav_tabs))))
}

// Returns uploader module in a turboframe.
pub async fn uploader_index() -> ResponseOk {
    ResponseOk(templates::Uploader { error: None }.render_once().unwrap())
}

// TODO: Figure out TempFile later
// #[post("/uploader", data = "<form>")]
// pub async fn uploader_upload(cluster: ConnectedCluster, form: Form<forms::Upload<'_>>) -> Result<Redirect, BadRequest> {
//     let mut uploaded_file = models::UploadedFile::create(cluster.pool()).await.unwrap();

//     match uploaded_file
//         .upload(cluster.pool(), form.file.path().unwrap(), form.has_header)
//         .await
//     {
//         Ok(()) => Ok(Redirect::to(format!(
//             "{}/done?table_name={}",
//             urls::deployment_uploader_turboframe(),
//             uploaded_file.table_name()
//         ))),
//         Err(err) => Err(BadRequest(
//             templates::Uploader {
//                 error: Some(err.to_string()),
//             }
//             .render_once()
//             .unwrap(),
//         )),
//     }
// }

#[derive(Deserialize)]
struct UploadedIndexParams {
    table_name: String,
}

pub async fn uploaded_index(
    cluster: ConnectedCluster,
    Query(UploadedIndexParams { table_name }): Query<UploadedIndexParams>,
) -> ResponseOk {
    let sql = templates::Sql::new(cluster.pool(), &format!("SELECT * FROM {table_name} LIMIT 10"))
        .await
        .unwrap();
    ResponseOk(
        templates::Uploaded {
            table_name: table_name.to_string(),
            columns: sql.columns.clone(),
            sql,
        }
        .render_once()
        .unwrap(),
    )
}
