use axum::{extract::Path, routing::get, Extension};
use sailfish::TemplateOnce;

use crate::{
    guards::{Cluster, ConnectedCluster},
    responses::{Error, ResponseOk},
    Router,
};

use crate::templates::{components::NavLink, *};

use crate::models;
use crate::templates;
use crate::utils::tabs;
use crate::utils::urls;
use std::collections::HashMap;

pub fn routes() -> Router {
    Router::new()
        .route("/snapshots", get(snapshots))
        .route("/snapshots/:snapshot_id", get(snapshot))
        .route("/snapshots_turboframe", get(snapshots_index))
        .route("/snapshots_turboframe/:id", get(snapshots_get))
}

// Returns snapshots page
pub async fn snapshots(
    Extension(cluster): Extension<Cluster>,
    _connected: ConnectedCluster,
) -> Result<ResponseOk, Error> {
    let mut layout = crate::templates::WebAppBase::new("Dashboard", &cluster);
    layout.breadcrumbs(vec![NavLink::new("Snapshots", &urls::deployment_snapshots()).active()]);

    let tabs = vec![tabs::Tab {
        name: "Snapshots",
        content: SnapshotsTab {}.render_once().unwrap(),
    }];

    let nav_tabs = tabs::Tabs::new(tabs, Some("Snapshots"), Some("Snapshots"))?;

    Ok(ResponseOk(layout.render(templates::Dashboard::new(nav_tabs))))
}

// Returns the specific snapshot page
pub async fn snapshot(
    Extension(cluster): Extension<Cluster>,
    Path(snapshot_id): Path<i64>,
    _connected: ConnectedCluster,
) -> Result<ResponseOk, Error> {
    let snapshot = models::Snapshot::get_by_id(cluster.pool(), snapshot_id).await?;

    let mut layout = crate::templates::WebAppBase::new("Dashboard", &cluster);
    layout.breadcrumbs(vec![
        NavLink::new("Snapshots", &urls::deployment_snapshots()),
        NavLink::new(&snapshot.relation_name, &urls::deployment_snapshot_by_id(snapshot.id)).active(),
    ]);

    let tabs = vec![tabs::Tab {
        name: "Snapshot",
        content: SnapshotTab { snapshot_id }.render_once().unwrap(),
    }];

    let nav_tabs = tabs::Tabs::new(tabs, Some("Snapshots"), Some("Snapshots"))?;

    Ok(ResponseOk(layout.render(templates::Dashboard::new(nav_tabs))))
}

// Returns all snapshots for the deployment in a turboframe.
pub async fn snapshots_index(cluster: ConnectedCluster) -> Result<ResponseOk, Error> {
    let snapshots = models::Snapshot::all(cluster.pool()).await?;

    Ok(ResponseOk(templates::Snapshots { snapshots }.render_once().unwrap()))
}

// Returns a specific snapshot for the deployment in a turboframe.
pub async fn snapshots_get(cluster: ConnectedCluster, Path(id): Path<i64>) -> Result<ResponseOk, Error> {
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
