use crate::forms;
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::route::Route;
use sailfish::TemplateOnce;

use crate::{
    guards::Cluster,
    guards::ConnectedCluster,
    responses::{BadRequest, Error, ResponseOk},
};

use crate::templates::{components::NavLink, *};

use crate::models;
use crate::templates;
use crate::utils::tabs;
use crate::utils::urls;

// Returns the uploader page.
#[get("/uploader")]
pub async fn uploader(cluster: &Cluster) -> Result<ResponseOk, Error> {
    let mut layout = crate::templates::WebAppBase::new("Dashboard", &cluster);
    layout.breadcrumbs(vec![NavLink::new("Upload Data", &urls::deployment_uploader()).active()]);

    let tabs = vec![tabs::Tab {
        name: "Upload data",
        content: UploaderTab { table_name: None }.render_once().unwrap(),
    }];

    let nav_tabs = tabs::Tabs::new(tabs, Some("Upload Data"), Some("Upload Data"))?;

    Ok(ResponseOk(layout.render(templates::Dashboard::new(nav_tabs, cluster))))
}

// Returns uploader module in a turboframe.
#[get("/uploader_turboframe")]
pub async fn uploader_index() -> ResponseOk {
    ResponseOk(templates::Uploader { error: None }.render_once().unwrap())
}

#[post("/uploader", data = "<form>")]
pub async fn uploader_upload(
    cluster: ConnectedCluster<'_>,
    form: Form<forms::Upload<'_>>,
) -> Result<Redirect, BadRequest> {
    let mut uploaded_file = models::UploadedFile::create(cluster.pool()).await.unwrap();

    match uploaded_file
        .upload(cluster.pool(), form.file.path().unwrap(), form.has_header)
        .await
    {
        Ok(()) => Ok(Redirect::to(format!(
            "{}/done?table_name={}",
            urls::deployment_uploader_turboframe(),
            uploaded_file.table_name()
        ))),
        Err(err) => Err(BadRequest(
            templates::Uploader {
                error: Some(err.to_string()),
            }
            .render_once()
            .unwrap(),
        )),
    }
}

#[get("/uploader_turboframe/done?<table_name>")]
pub async fn uploaded_index(cluster: ConnectedCluster<'_>, table_name: &str) -> ResponseOk {
    let sql = templates::Sql::new(cluster.pool(), &format!("SELECT * FROM {} LIMIT 10", table_name))
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

pub fn routes() -> Vec<Route> {
    routes![uploader, uploader_index, uploader_upload, uploaded_index,]
}
