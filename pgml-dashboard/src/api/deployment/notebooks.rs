use crate::forms;
use axum::{
    extract::{Path, Query},
    response::Redirect,
    routing::{get, post},
    Extension, Form, Json, Router,
};
use log::debug;
use sailfish::TemplateOnce;
use serde::Deserialize;

use crate::{
    guards::Cluster,
    guards::ConnectedCluster,
    responses::{Error, ResponseOk},
};

use crate::templates::{components::NavLink, *};
use crate::utils::tabs;

use crate::models;
use crate::templates;
use crate::utils::urls;

pub fn routes() -> Router {
    Router::new()
        .route("/notebooks", get(notebooks))
        .route("/notebooks/:id", get(notebook))
        .route("/notebooks_turboframe", get(notebook_index))
        .route("/notebooks", post(notebook_create))
        .route("/notebooks_turboframe/:notebook_id", get(notebook_get))
        .route("/notebooks/:notebook_id/reset", post(notebook_reset))
        .route("/notebooks/:notebook_id/cell", post(cell_create))
        .route("/notebooks/:notebook_id/reorder", post(notebook_reorder))
        .route("/notebooks/:notebook_id/cell/:cell_id/delete", post(cell_delete))
        .route("/notebooks/:notebook_id/cell/:cell_id/remove", post(cell_remove))
        .route("/notebooks/:notebook_id/cell/:cell_id/play", post(cell_play))
        .route("/notebooks/:notebook_id/cell/:cell_id/edit", get(cell_trigger_edit))
        .route("/notebooks/:notebook_id/cell/:cell_id/edit", post(cell_edit))
        .route("/notebooks/:notebook_id/cell/:cell_id", get(cell_get))
        .route("/notebooks/:notebook_id/cell/:cell_id/cancel", post(cell_cancel))
}

// Returns notebook page
pub async fn notebooks(
    Extension(cluster): Extension<Cluster>,
    _connected: ConnectedCluster,
) -> Result<ResponseOk, Error> {
    let mut layout = crate::templates::WebAppBase::new("Dashboard", &cluster);
    layout.breadcrumbs(vec![NavLink::new("Notebooks", &urls::deployment_notebooks()).active()]);

    let tabs = vec![tabs::Tab {
        name: "Notebooks",
        content: NotebooksTab {}.render_once().unwrap(),
    }];

    let nav_tabs = tabs::Tabs::new(tabs, Some("Notebooks"), Some("Notebooks"))?;

    Ok(ResponseOk(layout.render(templates::Dashboard::new(nav_tabs))))
}

// Returns the specified notebook page.
pub async fn notebook(
    Extension(cluster): Extension<Cluster>,
    Path(notebook_id): Path<i64>,
    _connected: ConnectedCluster,
) -> Result<ResponseOk, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;

    let mut layout = crate::templates::WebAppBase::new("Dashboard", &cluster);
    layout.breadcrumbs(vec![
        NavLink::new("Notebooks", &urls::deployment_notebooks()),
        NavLink::new(notebook.name.as_str(), &urls::deployment_notebook_by_id(notebook_id)).active(),
    ]);

    let tabs = vec![tabs::Tab {
        name: "Notebook",
        content: NotebookTab { id: notebook_id }.render_once().unwrap(),
    }];

    let nav_tabs = tabs::Tabs::new(tabs, Some("Notebooks"), Some("Notebooks"))?;

    Ok(ResponseOk(layout.render(templates::Dashboard::new(nav_tabs))))
}

#[derive(Deserialize)]
struct NotebookIndexParams {
    new: Option<String>,
}

// Returns all the notebooks for a deployment in a turbo frame.
pub async fn notebook_index(
    cluster: ConnectedCluster,
    Query(NotebookIndexParams { new }): Query<NotebookIndexParams>,
) -> Result<ResponseOk, Error> {
    Ok(ResponseOk(
        templates::Notebooks {
            notebooks: models::Notebook::all(cluster.pool()).await?,
            new: new.is_some(),
        }
        .render_once()
        .unwrap(),
    ))
}

#[derive(Deserialize)]
struct NotebookForm {
    name: String,
}

// Creates a new named notebook and redirects to that specific notebook.
pub async fn notebook_create(
    Extension(cluster): Extension<Cluster>,
    Form(data): Form<NotebookForm>,
) -> Result<Redirect, Error> {
    let notebook = crate::models::Notebook::create(cluster.pool(), &data.name).await?;

    models::Cell::create(cluster.pool(), &notebook, models::CellType::Sql as i32, "").await?;

    Ok(Redirect::to(&urls::deployment_notebook_by_id(notebook.id)))
}

// Returns the notebook in a turbo frame.
pub async fn notebook_get(cluster: ConnectedCluster, Path(notebook_id): Path<i64>) -> Result<ResponseOk, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let cells = notebook.cells(cluster.pool()).await?;

    Ok(ResponseOk(
        templates::Notebook { cells, notebook }.render_once().unwrap(),
    ))
}

pub async fn notebook_reset(cluster: ConnectedCluster, Path(notebook_id): Path<i64>) -> Result<Redirect, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    notebook.reset(cluster.pool()).await?;

    Ok(Redirect::to(&format!(
        "{}/{notebook_id}",
        urls::deployment_notebooks_turboframe(),
    )))
}

#[derive(Deserialize)]
struct CellForm {
    contents: String,
    cell_type: String,
}

pub async fn cell_create(
    cluster: ConnectedCluster,
    Path(notebook_id): Path<i64>,
    Form(cell): Form<CellForm>,
) -> Result<Redirect, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let mut cell = models::Cell::create(
        cluster.pool(),
        &notebook,
        cell.cell_type.parse::<i32>()?,
        &cell.contents,
    )
    .await?;

    if !cell.contents.is_empty() {
        cell.render(cluster.pool()).await?;
    }

    Ok(Redirect::to(&format!(
        "{}/{notebook_id}",
        urls::deployment_notebooks_turboframe(),
    )))
}

pub async fn notebook_reorder(
    cluster: ConnectedCluster,
    Path(notebook_id): Path<i64>,
    Json(cells): Json<forms::Reorder>,
) -> Result<Redirect, Error> {
    let _notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;

    let pool = cluster.pool();
    let mut transaction = pool.begin().await?;

    // Super bad n+1, but it's ok for now?
    for (idx, cell_id) in cells.cells.iter().enumerate() {
        let cell = models::Cell::get_by_id(&mut *transaction, *cell_id).await?;
        cell.reorder(&mut *transaction, idx as i32 + 1).await?;
    }

    transaction.commit().await?;

    Ok(Redirect::to(&format!(
        "{}/{notebook_id}",
        urls::deployment_notebooks_turboframe(),
    )))
}

pub async fn cell_get(
    cluster: ConnectedCluster,
    Path(notebook_id): Path<i64>,
    Path(cell_id): Path<i64>,
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

pub async fn cell_cancel(
    cluster: ConnectedCluster,
    Path(notebook_id): Path<i64>,
    Path(cell_id): Path<i64>,
) -> Result<Redirect, Error> {
    let cell = models::Cell::get_by_id(cluster.pool(), cell_id).await?;
    cell.cancel(cluster.pool()).await?;
    Ok(Redirect::to(&format!(
        "{}/{notebook_id}/cell/{cell_id}",
        urls::deployment_notebooks(),
    )))
}

pub async fn cell_edit(
    cluster: ConnectedCluster,
    Path(notebook_id): Path<i64>,
    Path(cell_id): Path<i64>,
    Form(data): Form<CellForm>,
) -> Result<ResponseOk, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let mut cell = models::Cell::get_by_id(cluster.pool(), cell_id).await?;

    cell.update(cluster.pool(), data.cell_type.parse::<i32>()?, &data.contents)
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

pub async fn cell_trigger_edit(
    cluster: ConnectedCluster,
    Path(notebook_id): Path<i64>,
    Path(cell_id): Path<i64>,
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

pub async fn cell_play(
    cluster: ConnectedCluster,
    Path(notebook_id): Path<i64>,
    Path(cell_id): Path<i64>,
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

pub async fn cell_remove(
    cluster: ConnectedCluster,
    Path(notebook_id): Path<i64>,
    Path(cell_id): Path<i64>,
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

pub async fn cell_delete(
    cluster: ConnectedCluster,
    Path(notebook_id): Path<i64>,
    Path(cell_id): Path<i64>,
) -> Result<Redirect, Error> {
    let _notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let cell = models::Cell::get_by_id(cluster.pool(), cell_id).await?;

    let _ = cell.delete(cluster.pool()).await?;

    Ok(Redirect::to(&format!(
        "{}/{notebook_id}/cell/{cell_id}",
        urls::deployment_notebooks(),
    )))
}
