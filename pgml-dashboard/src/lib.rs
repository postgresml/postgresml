#[macro_use]
extern crate rocket;

use std::collections::HashMap;

use rocket::form::Form;
use rocket::response::Redirect;
use rocket::route::Route;

use sqlx::{postgres::PgPoolOptions, PgPool};

use parking_lot::Mutex;
use sailfish::TemplateOnce;
use std::sync::Arc;

mod errors;
mod forms;
pub mod guards;
pub mod models;
mod responses;
mod templates;

use guards::Cluster;
use responses::{BadRequest, ResponseOk};

/// This struct contains information specific to the cluster being displayed in the dashboard.
///
/// The dashboard is built to manage multiple clusters, but the server itself by design is stateless.
/// This gives it a bit of shared state that allows the dashboard to display cluster-specific information.
#[derive(Debug, Default, Clone)]
pub struct Context {
    pub user: models::User,
    pub cluster: models::Cluster, 
}

/// Globally shared state, saved in memory.
///
/// If this state is reset, it should be trivial to rebuild it from a persistent medium, e.g. the database.
#[derive(Debug)]
pub struct Clusters {
    pools: Arc<Mutex<HashMap<i64, PgPool>>>,
    contexts: Arc<Mutex<HashMap<i64, Context>>>,
}

impl Clusters {
    pub fn add(&self, cluster_id: i64, database_url: &str) -> anyhow::Result<PgPool> {
        let mut pools = self.pools.lock();

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .idle_timeout(std::time::Duration::from_millis(15_000))
            .min_connections(0)
            .connect_lazy(database_url)?;

        pools.insert(cluster_id, pool.clone());

        Ok(pool)
    }

    /// Set the context for a cluster_id.
    ///
    ///This ideally should be set
    /// on every request to avoid stale cache.
    pub fn set_context(&self, cluster_id: i64, context: Context) {
        self.contexts.lock().insert(cluster_id, context);
    }

    /// Retrieve cluster context for the request.
    pub fn get_context(&self, cluster_id: i64) -> Context {
        match self.contexts.lock().get(&cluster_id) {
            Some(context) => context.clone(),
            None => Context::default(),
        }
    }

    /// Retrieve cluster connection pool reference.
    pub fn get(&self, cluster_id: i64) -> Option<PgPool> {
        match self.pools.lock().get(&cluster_id) {
            Some(pool) => Some(pool.clone()),
            None => None,
        }
    }

    /// Delete a cluster connection pool reference.
    pub fn delete(&self, cluster_id: i64) {
        let _ = self.pools.lock().remove(&cluster_id);
    }

    pub fn new() -> Clusters {
        Clusters {
            pools: Arc::new(Mutex::new(HashMap::new())),
            contexts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[get("/")]
pub async fn index() -> Redirect {
    Redirect::to("/dashboard/notebooks")
}

#[get("/projects")]
pub async fn project_index(cluster: Cluster) -> Result<ResponseOk, errors::Error> {
    Ok(ResponseOk(
        templates::Projects {
            topic: "projects".to_string(),
            projects: models::Project::all(cluster.pool()).await?,
            context: cluster.context.clone(),
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/projects/<id>")]
pub async fn project_get(cluster: Cluster, id: i64) -> Result<ResponseOk, errors::Error> {
    let project = models::Project::get_by_id(cluster.pool(), id).await?;
    let models = models::Model::get_by_project_id(cluster.pool(), id).await?;

    Ok(ResponseOk(
        templates::Project {
            topic: "projects".to_string(),
            project,
            models,
            context: cluster.context.clone(),
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/notebooks")]
pub async fn notebook_index(cluster: Cluster) -> Result<ResponseOk, errors::Error> {
    Ok(ResponseOk(
        templates::Notebooks {
            topic: "notebooks".to_string(),
            notebooks: models::Notebook::all(cluster.pool()).await?,
            context: cluster.context.clone(),
        }
        .render_once()
        .unwrap(),
    ))
}

#[post("/notebooks", data = "<data>")]
pub async fn notebook_create(
    cluster: Cluster,
    data: Form<forms::Notebook<'_>>,
) -> Result<Redirect, errors::Error> {
    let notebook = crate::models::Notebook::create(cluster.pool(), data.name).await?;

    Ok(Redirect::to(format!(
        "/dashboard/notebooks/{}/",
        notebook.id
    )))
}

#[get("/notebooks/<id>")]
pub async fn notebook_get(cluster: Cluster, id: i64) -> Result<ResponseOk, errors::Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), id).await?;

    Ok(ResponseOk(
        templates::Notebook {
            topic: "notebooks".to_string(),
            cells: notebook.cells(cluster.pool()).await?,
            notebook: notebook,
            context: cluster.context.clone(),
        }
        .render_once()
        .unwrap(),
    ))
}

#[post("/notebooks/<id>/reset")]
pub async fn notebook_reset(cluster: Cluster, id: i64) -> Result<Redirect, errors::Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), id).await?;
    notebook.reset(cluster.pool()).await?;

    Ok(Redirect::to(format!("/dashboard/notebooks/{}", id)))
}

#[post("/notebooks/<id>/cell", data = "<cell>")]
pub async fn cell_create(
    cluster: Cluster,
    id: i64,
    cell: Form<forms::Cell<'_>>,
) -> Result<Redirect, errors::Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), id).await?;
    let mut cell = models::Cell::create(
        cluster.pool(),
        &notebook,
        cell.cell_type.parse::<i32>()?,
        cell.contents,
    )
    .await?;
    let _ = cell.render(cluster.pool()).await?;

    Ok(Redirect::to(format!("/dashboard/notebooks/{}/", id)))
}

#[get("/notebooks/<notebook_id>/cell/<cell_id>")]
pub async fn cell_get(
    cluster: Cluster,
    notebook_id: i64,
    cell_id: i64,
) -> Result<ResponseOk, errors::Error> {
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
    cluster: Cluster,
    notebook_id: i64,
    cell_id: i64,
    data: Form<forms::Cell<'_>>,
) -> Result<ResponseOk, errors::Error> {
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
    cluster: Cluster,
    notebook_id: i64,
    cell_id: i64,
) -> Result<ResponseOk, errors::Error> {
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
    cluster: Cluster,
    notebook_id: i64,
    cell_id: i64,
) -> Result<ResponseOk, errors::Error> {
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
    cluster: Cluster,
    notebook_id: i64,
    cell_id: i64,
) -> Result<ResponseOk, errors::Error> {
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
    cluster: Cluster,
    notebook_id: i64,
    cell_id: i64,
) -> Result<Redirect, errors::Error> {
    let _notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let cell = models::Cell::get_by_id(cluster.pool(), cell_id).await?;

    let _ = cell.delete(cluster.pool()).await?;

    Ok(Redirect::to(format!(
        "/dashboard/notebooks/{}",
        notebook_id
    )))
}

#[get("/models")]
pub async fn models_index(cluster: Cluster) -> Result<ResponseOk, errors::Error> {
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
            topic: "models".to_string(),
            projects,
            models,
            context: cluster.context.clone(),
            // min_scores,
            // max_scores,
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/models/<id>")]
pub async fn models_get(cluster: Cluster, id: i64) -> Result<ResponseOk, errors::Error> {
    let model = models::Model::get_by_id(cluster.pool(), id).await?;
    let snapshot = models::Snapshot::get_by_id(cluster.pool(), model.snapshot_id).await?;
    let project = models::Project::get_by_id(cluster.pool(), model.project_id).await?;

    Ok(ResponseOk(
        templates::Model {
            topic: "models".to_string(),
            deployed: model.deployed(cluster.pool()).await?,
            model,
            snapshot,
            project,
            context: cluster.context.clone(),
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/snapshots")]
pub async fn snapshots_index(cluster: Cluster) -> Result<ResponseOk, errors::Error> {
    let snapshots = models::Snapshot::all(cluster.pool()).await?;
    let mut table_sizes = HashMap::new();
    for snapshot in &snapshots {
        let table_size = snapshot.table_size(cluster.pool()).await?;
        table_sizes.insert(snapshot.id, table_size);
    }

    Ok(ResponseOk(
        templates::Snapshots {
            topic: "snapshots".to_string(),
            snapshots,
            table_sizes,
            context: cluster.context.clone(),
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/snapshots/<id>")]
pub async fn snapshots_get(cluster: Cluster, id: i64) -> Result<ResponseOk, errors::Error> {
    let snapshot = models::Snapshot::get_by_id(cluster.pool(), id).await?;
    let samples = snapshot.samples(cluster.pool(), 500).await?;
    let models = snapshot.models(cluster.pool()).await?;
    let mut projects = HashMap::new();

    for model in &models {
        projects.insert(model.project_id, model.project(cluster.pool()).await?);
    }

    Ok(ResponseOk(
        templates::Snapshot {
            topic: "snapshots".to_string(),
            table_size: snapshot.table_size(cluster.pool()).await?,
            snapshot,
            models,
            projects,
            samples,
            context: cluster.context.clone(),
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/deployments")]
pub async fn deployments_index(cluster: Cluster) -> Result<ResponseOk, errors::Error> {
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
            topic: "deployments".to_string(),
            projects,
            deployments,
            context: cluster.context.clone(),
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/deployments/<id>")]
pub async fn deployments_get(cluster: Cluster, id: i64) -> Result<ResponseOk, errors::Error> {
    let deployment = models::Deployment::get_by_id(cluster.pool(), id).await?;
    let project = models::Project::get_by_id(cluster.pool(), deployment.project_id).await?;
    let model = models::Model::get_by_id(cluster.pool(), deployment.model_id).await?;

    Ok(ResponseOk(
        templates::Deployment {
            topic: "deployments".to_string(),
            project,
            deployment,
            model,
            context: cluster.context.clone(),
        }
        .render_once()
        .unwrap(),
    ))
}

#[get("/uploader")]
pub async fn uploader_index(cluster: Cluster) -> ResponseOk {
    ResponseOk(
        templates::Uploader {
            topic: "uploader".to_string(),
            error: None,
            context: cluster.context.clone(),
        }
        .render_once()
        .unwrap(),
    )
}

#[post("/uploader", data = "<form>")]
pub async fn uploader_upload(
    cluster: Cluster,
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
                topic: "uploader".to_string(),
                error: Some(err.to_string()),
                context: cluster.context.clone(),
            }
            .render_once()
            .unwrap(),
        )),
    }
}

#[get("/uploader/done?<table_name>")]
pub async fn uploaded_index(cluster: Cluster, table_name: &str) -> ResponseOk {
    let sql = templates::Sql::new(
        cluster.pool(),
        &format!("SELECT * FROM {} LIMIT 10", table_name),
    )
    .await
    .unwrap();
    ResponseOk(
        templates::Uploaded {
            topic: "uploader".to_string(),
            table_name: table_name.to_string(),
            columns: sql.columns.clone(),
            sql,
            context: cluster.context.clone(),
        }
        .render_once()
        .unwrap(),
    )
}

pub fn paths() -> Vec<Route> {
    routes![
        index,
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
    ]
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    Ok(sqlx::migrate!("./migrations").run(pool).await?)
}
