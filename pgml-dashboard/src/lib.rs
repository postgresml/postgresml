#[macro_use]
extern crate rocket;

use rocket::form::Form;
use rocket::response::Redirect;
use rocket::route::Route;
use rocket::serde::json::Json;
use sailfish::TemplateOnce;
use sqlx::PgPool;
use std::collections::HashMap;

pub mod api;
pub mod components;
pub mod fairings;
pub mod forms;
pub mod guards;
pub mod models;
pub mod responses;
pub mod templates;
pub mod utils;

use guards::{Cluster, ConnectedCluster};
use responses::{BadRequest, Error, ResponseOk};
use templates::{
    components::{NavLink, StaticNav},
    *,
};
use utils::tabs;

#[derive(Debug, Default, Clone)]
pub struct ClustersSettings {
    pub max_connections: u32,
    pub idle_timeout: u64,
    pub min_connections: u32,
}

/// This struct contains information specific to the cluster being displayed in the dashboard.
///
/// The dashboard is built to manage multiple clusters, but the server itself by design is stateless.
/// This gives it a bit of shared state that allows the dashboard to display cluster-specific information.
#[derive(Debug, Default, Clone)]
pub struct Context {
    pub user: models::User,
    pub cluster: models::Cluster,
    pub dropdown_nav: StaticNav,
    pub account_management_nav: StaticNav,
    pub upper_left_nav: StaticNav,
    pub lower_left_nav: StaticNav,
}

#[get("/projects")]
pub async fn project_index(cluster: ConnectedCluster<'_>) -> Result<ResponseOk, Error> {
    Ok(ResponseOk(
        templates::Projects {
            projects: models::Project::all(cluster.pool()).await?,
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/projects/<id>")]
pub async fn project_get(cluster: ConnectedCluster<'_>, id: i64) -> Result<ResponseOk, Error> {
    let project = models::Project::get_by_id(cluster.pool(), id).await?;
    let models = models::Model::get_by_project_id(cluster.pool(), id).await?;

    Ok(ResponseOk(
        templates::Project { project, models }
            .render_once()
            .unwrap(),
    ))
}

#[get("/notebooks?<new>")]
pub async fn notebook_index(
    cluster: ConnectedCluster<'_>,
    new: Option<&str>,
) -> Result<ResponseOk, Error> {
    Ok(ResponseOk(
        templates::Notebooks {
            notebooks: models::Notebook::all(&cluster.pool()).await?,
            new: new.is_some(),
        }
        .render_once()
        .unwrap(),
    ))
}

#[post("/notebooks", data = "<data>")]
pub async fn notebook_create(
    cluster: &Cluster,
    data: Form<forms::Notebook<'_>>,
) -> Result<Redirect, Error> {
    let notebook = crate::models::Notebook::create(cluster.pool(), data.name).await?;

    models::Cell::create(cluster.pool(), &notebook, models::CellType::Sql as i32, "").await?;

    Ok(Redirect::to(format!(
        "/dashboard?tab=Notebook&id={}",
        notebook.id
    )))
}

#[get("/notebooks/<notebook_id>")]
pub async fn notebook_get(
    cluster: ConnectedCluster<'_>,
    notebook_id: i64,
) -> Result<ResponseOk, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let cells = notebook.cells(cluster.pool()).await?;

    Ok(ResponseOk(
        templates::Notebook { cells, notebook }
            .render_once()
            .unwrap(),
    ))
}

#[post("/notebooks/<notebook_id>/reset")]
pub async fn notebook_reset(
    cluster: ConnectedCluster<'_>,
    notebook_id: i64,
) -> Result<Redirect, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    notebook.reset(cluster.pool()).await?;

    Ok(Redirect::to(format!(
        "/dashboard/notebooks/{}",
        notebook_id
    )))
}

#[post("/notebooks/<notebook_id>/cell", data = "<cell>")]
pub async fn cell_create(
    cluster: ConnectedCluster<'_>,
    notebook_id: i64,
    cell: Form<forms::Cell<'_>>,
) -> Result<Redirect, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let mut cell = models::Cell::create(
        cluster.pool(),
        &notebook,
        cell.cell_type.parse::<i32>()?,
        cell.contents,
    )
    .await?;

    if !cell.contents.is_empty() {
        let _ = cell.render(cluster.pool()).await?;
    }

    Ok(Redirect::to(format!(
        "/dashboard/notebooks/{}",
        notebook_id
    )))
}

#[post("/notebooks/<notebook_id>/reorder", data = "<cells>")]
pub async fn notebook_reorder(
    cluster: ConnectedCluster<'_>,
    notebook_id: i64,
    cells: Json<forms::Reorder>,
) -> Result<Redirect, Error> {
    let _notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;

    let pool = cluster.pool();
    let mut transaction = pool.begin().await?;

    // Super bad n+1, but it's ok for now?
    for (idx, cell_id) in cells.cells.iter().enumerate() {
        let cell = models::Cell::get_by_id(&mut transaction, *cell_id).await?;
        cell.reorder(&mut transaction, idx as i32 + 1).await?;
    }

    transaction.commit().await?;

    Ok(Redirect::to(format!(
        "/dashboard/notebooks/{}",
        notebook_id
    )))
}

#[get("/notebooks/<notebook_id>/cell/<cell_id>")]
pub async fn cell_get(
    cluster: ConnectedCluster<'_>,
    notebook_id: i64,
    cell_id: i64,
) -> Result<ResponseOk, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let cell = models::Cell::get_by_id(cluster.pool(), cell_id).await?;

    Ok(ResponseOk(
        templates::Cell {
            cell,
            notebook,
            selected: false,
            edit: false,
        }
        .render_once()
        .unwrap(),
    ))
}

#[post("/notebooks/<notebook_id>/cell/<cell_id>/cancel")]
pub async fn cell_cancel(
    cluster: ConnectedCluster<'_>,
    notebook_id: i64,
    cell_id: i64,
) -> Result<Redirect, Error> {
    let cell = models::Cell::get_by_id(cluster.pool(), cell_id).await?;
    cell.cancel(cluster.pool()).await?;
    Ok(Redirect::to(format!(
        "/dashboard/notebooks/{}/cell/{}",
        notebook_id, cell_id
    )))
}

#[post("/notebooks/<notebook_id>/cell/<cell_id>/edit", data = "<data>")]
pub async fn cell_edit(
    cluster: ConnectedCluster<'_>,
    notebook_id: i64,
    cell_id: i64,
    data: Form<forms::Cell<'_>>,
) -> Result<ResponseOk, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let mut cell = models::Cell::get_by_id(cluster.pool(), cell_id).await?;

    cell.update(
        cluster.pool(),
        data.cell_type.parse::<i32>()?,
        &data.contents,
    )
    .await?;

    debug!("Rendering cell id={}", cell.id);
    cell.render(cluster.pool()).await?;
    debug!("Rendering of cell id={} complete", cell.id);

    Ok(ResponseOk(
        templates::Cell {
            cell,
            notebook,
            selected: false,
            edit: false,
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/notebooks/<notebook_id>/cell/<cell_id>/edit")]
pub async fn cell_trigger_edit(
    cluster: ConnectedCluster<'_>,
    notebook_id: i64,
    cell_id: i64,
) -> Result<ResponseOk, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let cell = models::Cell::get_by_id(cluster.pool(), cell_id).await?;

    Ok(ResponseOk(
        templates::Cell {
            cell,
            notebook,
            selected: true,
            edit: true,
        }
        .render_once()
        .unwrap(),
    ))
}

#[post("/notebooks/<notebook_id>/cell/<cell_id>/play")]
pub async fn cell_play(
    cluster: ConnectedCluster<'_>,
    notebook_id: i64,
    cell_id: i64,
) -> Result<ResponseOk, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let mut cell = models::Cell::get_by_id(cluster.pool(), cell_id).await?;
    cell.render(cluster.pool()).await?;

    Ok(ResponseOk(
        templates::Cell {
            cell,
            notebook,
            selected: true,
            edit: false,
        }
        .render_once()
        .unwrap(),
    ))
}

#[post("/notebooks/<notebook_id>/cell/<cell_id>/remove")]
pub async fn cell_remove(
    cluster: ConnectedCluster<'_>,
    notebook_id: i64,
    cell_id: i64,
) -> Result<ResponseOk, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let cell = models::Cell::get_by_id(cluster.pool(), cell_id).await?;
    let bust_cache = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)?
        .as_millis()
        .to_string();

    Ok(ResponseOk(
        templates::Undo {
            notebook,
            cell,
            bust_cache,
        }
        .render_once()?,
    ))
}

#[post("/notebooks/<notebook_id>/cell/<cell_id>/delete")]
pub async fn cell_delete(
    cluster: ConnectedCluster<'_>,
    notebook_id: i64,
    cell_id: i64,
) -> Result<Redirect, Error> {
    let _notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let cell = models::Cell::get_by_id(cluster.pool(), cell_id).await?;

    let _ = cell.delete(cluster.pool()).await?;

    Ok(Redirect::to(format!(
        "/dashboard/notebooks/{}/cell/{}",
        notebook_id, cell_id
    )))
}

#[get("/models")]
pub async fn models_index(cluster: ConnectedCluster<'_>) -> Result<ResponseOk, Error> {
    let projects = models::Project::all(cluster.pool()).await?;
    let mut models = HashMap::new();
    // let mut max_scores = HashMap::new();
    // let mut min_scores = HashMap::new();

    for project in &projects {
        let project_models = models::Model::get_by_project_id(cluster.pool(), project.id).await?;
        // let mut key_metrics = project_models
        //     .iter()
        //     .map(|m| m.key_metric(project).unwrap_or(0.))
        //     .collect::<Vec<f64>>();
        // key_metrics.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // max_scores.insert(project.id, key_metrics.iter().last().unwrap_or(&0.).clone());
        // min_scores.insert(project.id, key_metrics.iter().next().unwrap_or(&0.).clone());

        models.insert(project.id, project_models);
    }

    Ok(ResponseOk(
        templates::Models {
            projects,
            models,
            // min_scores,
            // max_scores,
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/models/<id>")]
pub async fn models_get(cluster: ConnectedCluster<'_>, id: i64) -> Result<ResponseOk, Error> {
    let model = models::Model::get_by_id(cluster.pool(), id).await?;
    let snapshot = if let Some(snapshot_id) = model.snapshot_id {
        Some(models::Snapshot::get_by_id(cluster.pool(), snapshot_id).await?)
    } else {
        None
    };

    let project = models::Project::get_by_id(cluster.pool(), model.project_id).await?;

    Ok(ResponseOk(
        templates::Model {
            deployed: model.deployed(cluster.pool()).await?,
            model,
            snapshot,
            project,
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/snapshots")]
pub async fn snapshots_index(cluster: ConnectedCluster<'_>) -> Result<ResponseOk, Error> {
    let snapshots = models::Snapshot::all(cluster.pool()).await?;

    Ok(ResponseOk(
        templates::Snapshots { snapshots }.render_once().unwrap(),
    ))
}

#[get("/snapshots/<id>")]
pub async fn snapshots_get(cluster: ConnectedCluster<'_>, id: i64) -> Result<ResponseOk, Error> {
    let snapshot = models::Snapshot::get_by_id(cluster.pool(), id).await?;
    let samples = snapshot.samples(cluster.pool(), 500).await?;

    let models = snapshot.models(cluster.pool()).await?;
    let mut projects = HashMap::new();

    for model in &models {
        projects.insert(model.project_id, model.project(cluster.pool()).await?);
    }

    Ok(ResponseOk(
        templates::Snapshot {
            snapshot,
            models,
            projects,
            samples,
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/deployments")]
pub async fn deployments_index(cluster: ConnectedCluster<'_>) -> Result<ResponseOk, Error> {
    let projects = models::Project::all(cluster.pool()).await?;
    let mut deployments = HashMap::new();

    for project in projects.iter() {
        deployments.insert(
            project.id,
            models::Deployment::get_by_project_id(cluster.pool(), project.id).await?,
        );
    }

    Ok(ResponseOk(
        templates::Deployments {
            projects,
            deployments,
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/deployments/<id>")]
pub async fn deployments_get(cluster: ConnectedCluster<'_>, id: i64) -> Result<ResponseOk, Error> {
    let deployment = models::Deployment::get_by_id(cluster.pool(), id).await?;
    let project = models::Project::get_by_id(cluster.pool(), deployment.project_id).await?;
    let model = models::Model::get_by_id(cluster.pool(), deployment.model_id).await?;

    Ok(ResponseOk(
        templates::Deployment {
            project,
            deployment,
            model,
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/uploader")]
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
            "/dashboard/uploader/done?table_name={}",
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

#[get("/uploader/done?<table_name>")]
pub async fn uploaded_index(cluster: ConnectedCluster<'_>, table_name: &str) -> ResponseOk {
    let sql = templates::Sql::new(
        cluster.pool(),
        &format!("SELECT * FROM {} LIMIT 10", table_name),
    )
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

#[get("/?<tab>&<id>")]
pub async fn dashboard(
    cluster: ConnectedCluster<'_>,
    tab: Option<&str>,
    id: Option<i64>,
) -> Result<ResponseOk, Error> {
    let mut layout = crate::templates::WebAppBase::new("Dashboard", &cluster.inner.context);

    let mut breadcrumbs = vec![NavLink::new("Dashboard", "/dashboard")];

    let tab = tab.unwrap_or("Notebooks");

    match tab {
        "Notebooks" => {
            breadcrumbs.push(NavLink::new("Notebooks", "/dashboard?tab=Notebooks").active());
        }

        "Notebook" => {
            let notebook = models::Notebook::get_by_id(cluster.pool(), id.unwrap()).await?;
            breadcrumbs.push(NavLink::new("Notebooks", "/dashboard?tab=Notebooks"));

            breadcrumbs.push(
                NavLink::new(
                    notebook.name.as_str(),
                    &format!("/dashboard?tab=Notebook&id={}", notebook.id),
                )
                .active(),
            );
        }

        "Projects" => {
            breadcrumbs.push(NavLink::new("Projects", "/dashboard?tab=Projects").active());
        }

        "Project" => {
            let project = models::Project::get_by_id(cluster.pool(), id.unwrap()).await?;
            breadcrumbs.push(NavLink::new("Projects", "/dashboard?tab=Projects"));
            breadcrumbs.push(
                NavLink::new(
                    &project.name,
                    &format!("/dashboard?tab=Project&id={}", project.id),
                )
                .active(),
            );
        }

        "Models" => {
            breadcrumbs.push(NavLink::new("Models", "/dashboard?tab=Models").active());
        }

        "Model" => {
            let model = models::Model::get_by_id(cluster.pool(), id.unwrap()).await?;
            let project = models::Project::get_by_id(cluster.pool(), model.project_id).await?;

            breadcrumbs.push(NavLink::new("Models", "/dashboard?tab=Models"));
            breadcrumbs.push(NavLink::new(
                &project.name,
                &format!("/dashboard?tab=Project&id={}", project.id),
            ));
            breadcrumbs.push(
                NavLink::new(
                    &model.algorithm,
                    &format!("/dashboard?tab=Model&id={}", model.id),
                )
                .active(),
            );
        }

        "Snapshots" => {
            breadcrumbs.push(NavLink::new("Snapshots", "/dashboard?tab=Snapshots").active());
        }

        "Snapshot" => {
            let snapshot = models::Snapshot::get_by_id(cluster.pool(), id.unwrap()).await?;

            breadcrumbs.push(NavLink::new("Snapshots", "/dashboard?tab=Snapshots"));
            breadcrumbs.push(
                NavLink::new(
                    &snapshot.relation_name,
                    &format!("/dashboard?tab=Snapshot&id={}", snapshot.id),
                )
                .active(),
            );
        }

        "Upload_Data" => {
            breadcrumbs.push(NavLink::new("Upload Data", "/dashboard?tab=Upload_Data").active());
        }
        _ => (),
    };

    layout.breadcrumbs(breadcrumbs);

    let tabs = match tab {
        "Notebooks" => vec![tabs::Tab {
            name: "Notebooks",
            content: NotebooksTab {}.render_once().unwrap(),
        }],
        "Projects" => vec![tabs::Tab {
            name: "Projects",
            content: ProjectsTab {}.render_once().unwrap(),
        }],
        "Notebook" => vec![tabs::Tab {
            name: "Notebook",
            content: NotebookTab { id: id.unwrap() }.render_once().unwrap(),
        }],
        "Project" => vec![tabs::Tab {
            name: "Project",
            content: ProjectTab {
                project_id: id.unwrap(),
            }
            .render_once()
            .unwrap(),
        }],
        "Models" => vec![tabs::Tab {
            name: "Models",
            content: ModelsTab {}.render_once().unwrap(),
        }],

        "Model" => vec![tabs::Tab {
            name: "Model",
            content: ModelTab {
                model_id: id.unwrap(),
            }
            .render_once()
            .unwrap(),
        }],

        "Snapshots" => vec![tabs::Tab {
            name: "Snapshots",
            content: SnapshotsTab {}.render_once().unwrap(),
        }],

        "Snapshot" => vec![tabs::Tab {
            name: "Snapshot",
            content: SnapshotTab {
                snapshot_id: id.unwrap(),
            }
            .render_once()
            .unwrap(),
        }],

        "Upload_Data" => vec![tabs::Tab {
            name: "Upload data",
            content: UploaderTab { table_name: None }.render_once().unwrap(),
        }],
        _ => todo!(),
    };

    let nav_tabs = tabs::Tabs::new(tabs, Some("Notebooks"), Some(tab))?;

    Ok(ResponseOk(
        layout.render(templates::Dashboard { tabs: nav_tabs }),
    ))
}

#[get("/playground")]
pub async fn playground(cluster: &Cluster) -> Result<ResponseOk, Error> {
    let mut layout = crate::templates::WebAppBase::new("Playground", &cluster.context);
    Ok(ResponseOk(layout.render(templates::Playground {})))
}

#[get("/chatbot")]
pub async fn chatbot(cluster: &Cluster) -> Result<ResponseOk, Error> {
    let mut layout = crate::templates::WebAppBase::new("Chatbot", &cluster.context);
    Ok(ResponseOk(layout.render(templates::Chatbot {})))
}

pub fn routes() -> Vec<Route> {
    routes![
        notebook_index,
        project_index,
        project_get,
        notebook_create,
        notebook_get,
        notebook_reset,
        cell_create,
        cell_get,
        cell_trigger_edit,
        cell_edit,
        cell_play,
        cell_remove,
        cell_delete,
        cell_cancel,
        models_index,
        models_get,
        snapshots_index,
        snapshots_get,
        deployments_index,
        deployments_get,
        uploader_index,
        uploader_upload,
        uploaded_index,
        dashboard,
        notebook_reorder,
    ]
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    Ok(sqlx::migrate!("./migrations").run(pool).await?)
}
