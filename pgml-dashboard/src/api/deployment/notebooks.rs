use crate::forms;
use sailfish::TemplateOnce;

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

// Returns notebook page
#[get("/notebooks")]
pub async fn notebooks(cluster: &Cluster, _connected: ConnectedCluster<'_>) -> Result<ResponseOk, Error> {
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
#[get("/notebooks/<notebook_id>")]
pub async fn notebook(
    cluster: &Cluster,
    notebook_id: i64,
    _connected: ConnectedCluster<'_>,
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

// Returns all the notebooks for a deployment in a turbo frame.
#[get("/notebooks_turboframe?<new>")]
pub async fn notebook_index(cluster: ConnectedCluster<'_>, new: Option<&str>) -> Result<ResponseOk, Error> {
    Ok(ResponseOk(
        templates::Notebooks {
            notebooks: models::Notebook::all(cluster.pool()).await?,
            new: new.is_some(),
        }
        .render_once()
        .unwrap(),
    ))
}

// Creates a new named notebook and redirects to that specific notebook.
#[post("/notebooks", data = "<data>")]
pub async fn notebook_create(cluster: &Cluster, data: Form<forms::Notebook<'_>>) -> Result<Redirect, Error> {
    let notebook = crate::models::Notebook::create(cluster.pool(), data.name).await?;

    models::Cell::create(cluster.pool(), &notebook, models::CellType::Sql as i32, "").await?;

    Ok(Redirect::to(urls::deployment_notebook_by_id(notebook.id)))
}

// Returns the notebook in a turbo frame.
#[get("/notebooks_turboframe/<notebook_id>")]
pub async fn notebook_get(cluster: ConnectedCluster<'_>, notebook_id: i64) -> Result<ResponseOk, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let cells = notebook.cells(cluster.pool()).await?;

    Ok(ResponseOk(
        templates::Notebook { cells, notebook }.render_once().unwrap(),
    ))
}

#[post("/notebooks/<notebook_id>/reset")]
pub async fn notebook_reset(cluster: ConnectedCluster<'_>, notebook_id: i64) -> Result<Redirect, Error> {
    let notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    notebook.reset(cluster.pool()).await?;

    Ok(Redirect::to(format!(
        "{}/{}",
        urls::deployment_notebooks_turboframe(),
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
    let mut cell =
        models::Cell::create(cluster.pool(), &notebook, cell.cell_type.parse::<i32>()?, cell.contents).await?;

    if !cell.contents.is_empty() {
        cell.render(cluster.pool()).await?;
    }

    Ok(Redirect::to(format!(
        "{}/{}",
        urls::deployment_notebooks_turboframe(),
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
        let cell = models::Cell::get_by_id(&mut *transaction, *cell_id).await?;
        cell.reorder(&mut *transaction, idx as i32 + 1).await?;
    }

    transaction.commit().await?;

    Ok(Redirect::to(format!(
        "{}/{}",
        urls::deployment_notebooks_turboframe(),
        notebook_id
    )))
}

#[get("/notebooks/<notebook_id>/cell/<cell_id>")]
pub async fn cell_get(cluster: ConnectedCluster<'_>, notebook_id: i64, cell_id: i64) -> Result<ResponseOk, Error> {
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
pub async fn cell_cancel(cluster: ConnectedCluster<'_>, notebook_id: i64, cell_id: i64) -> Result<Redirect, Error> {
    let cell = models::Cell::get_by_id(cluster.pool(), cell_id).await?;
    cell.cancel(cluster.pool()).await?;
    Ok(Redirect::to(format!(
        "{}/{}/cell/{}",
        urls::deployment_notebooks(),
        notebook_id,
        cell_id
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

    cell.update(cluster.pool(), data.cell_type.parse::<i32>()?, data.contents)
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
pub async fn cell_play(cluster: ConnectedCluster<'_>, notebook_id: i64, cell_id: i64) -> Result<ResponseOk, Error> {
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
pub async fn cell_remove(cluster: ConnectedCluster<'_>, notebook_id: i64, cell_id: i64) -> Result<ResponseOk, Error> {
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
pub async fn cell_delete(cluster: ConnectedCluster<'_>, notebook_id: i64, cell_id: i64) -> Result<Redirect, Error> {
    let _notebook = models::Notebook::get_by_id(cluster.pool(), notebook_id).await?;
    let cell = models::Cell::get_by_id(cluster.pool(), cell_id).await?;

    let _ = cell.delete(cluster.pool()).await?;

    Ok(Redirect::to(format!(
        "{}/{}/cell/{}",
        urls::deployment_notebooks(),
        notebook_id,
        cell_id
    )))
}

pub fn routes() -> Vec<Route> {
    routes![
        notebooks,
        notebook,
        notebook_index,
        notebook_create,
        notebook_get,
        notebook_reset,
        cell_create,
        notebook_reorder,
        cell_get,
        cell_cancel,
        cell_edit,
        cell_trigger_edit,
        cell_play,
        cell_remove,
        cell_delete
    ]
}
