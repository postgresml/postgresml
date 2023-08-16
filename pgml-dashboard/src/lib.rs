#[macro_use]
extern crate rocket;

use rocket::form::Form;
use rocket::response::Redirect;
use rocket::route::Route;
use sailfish::TemplateOnce;
use sqlx::PgPool;
use std::collections::HashMap;

pub mod api;
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
    components::StaticNav, DeploymentsTab, Layout, ModelsTab, NotebooksTab, ProjectsTab,
    SnapshotsTab, UploaderTab,
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

#[get("/notebooks")]
pub async fn notebook_index(cluster: ConnectedCluster<'_>) -> Result<ResponseOk, Error> {
    Ok(ResponseOk(
        templates::Notebooks {
            notebooks: models::Notebook::all(&cluster.pool()).await?,
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

    Ok(Redirect::to(format!(
        "/dashboard?tab=Notebooks&notebook_id={}",
        notebook.id
    )))
}

#[get("/notebooks/<notebook_id>")]
pub async fn notebook_get(
    cluster: ConnectedCluster<'_>,
    notebook_id: i64,
) -> Result<ResponseOk, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;

    Ok(ResponseOk(Layout::new("Notebooks").render(
        templates::Notebook {
            cells: notebook.cells(cluster.pool()).await?,
            notebook,
        },
    )))
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
    cluster: &Cluster,
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
    let _ = cell.render(cluster.pool()).await?;

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

    let bust_cache = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)?
        .as_millis()
        .to_string();

    Ok(ResponseOk(
        templates::Cell {
            cell,
            notebook,
            selected: false,
            edit: false,
            bust_cache,
        }
        .render_once()
        .unwrap(),
    ))
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
    cell.render(cluster.pool()).await?;

    let bust_cache = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)?
        .as_millis()
        .to_string();

    Ok(ResponseOk(
        templates::Cell {
            cell,
            notebook,
            selected: false,
            edit: false,
            bust_cache,
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
    let bust_cache = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)?
        .as_millis()
        .to_string();

    Ok(ResponseOk(
        templates::Cell {
            cell,
            notebook,
            selected: false,
            edit: true,
            bust_cache,
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
    let bust_cache = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)?
        .as_millis()
        .to_string();

    Ok(ResponseOk(
        templates::Cell {
            cell,
            notebook,
            selected: true,
            edit: false,
            bust_cache,
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
        "/dashboard/notebooks/{}",
        notebook_id
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
    let snapshot = models::Snapshot::get_by_id(cluster.pool(), model.snapshot_id).await?;
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
            "/dashboard?tab=Upload_Data&table_name={}",
            uploaded_file.table_name()
        ))),
        Err(err) => Err(BadRequest(Layout::new("Uploader").render(
            templates::Uploader {
                error: Some(err.to_string()),
            },
        ))),
    }
}

#[get("/uploader/done?<table_name>")]
pub async fn uploaded_index(cluster: ConnectedCluster<'_>, table_name: &str) -> ResponseOk {
    let sql = templates::Sql::new(
        cluster.pool(),
        &format!("SELECT * FROM {} LIMIT 10", table_name),
        true,
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

#[get("/?<tab>&<notebook_id>&<model_id>&<project_id>&<snapshot_id>&<deployment_id>&<table_name>")]
pub async fn dashboard(
    cluster: &Cluster,
    tab: Option<&str>,
    notebook_id: Option<i64>,
    model_id: Option<i64>,
    project_id: Option<i64>,
    snapshot_id: Option<i64>,
    deployment_id: Option<i64>,
    table_name: Option<String>,
) -> Result<ResponseOk, Error> {
    let mut layout = crate::templates::WebAppBase::new("Dashboard", &cluster.context);
    layout.breadcrumbs(vec![crate::templates::components::NavLink::new(
        "Dashboard",
        "/dashboard",
    )
    .active()]);

    let all_tabs = vec![
        tabs::Tab {
            name: "Notebooks",
            content: NotebooksTab { notebook_id }.render_once().unwrap(),
        },
        tabs::Tab {
            name: "Projects",
            content: ProjectsTab { project_id }.render_once().unwrap(),
        },
        tabs::Tab {
            name: "Models",
            content: ModelsTab { model_id }.render_once().unwrap(),
        },
        tabs::Tab {
            name: "Deployments",
            content: DeploymentsTab { deployment_id }.render_once().unwrap(),
        },
        tabs::Tab {
            name: "Snapshots",
            content: SnapshotsTab { snapshot_id }.render_once().unwrap(),
        },
        tabs::Tab {
            name: "Upload_Data",
            content: UploaderTab { table_name }.render_once().unwrap(),
        },
    ];

    let nav_tabs = tabs::Tabs::new(all_tabs, Some("Notebooks"), tab)?;

    Ok(ResponseOk(
        layout.render(templates::Dashboard { tabs: nav_tabs }),
    ))
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
    ]
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    Ok(sqlx::migrate!("./migrations").run(pool).await?)
}
