use rocket::route::Route;
use sailfish::TemplateOnce;

use crate::{
    guards::Cluster,
    guards::ConnectedCluster,
    responses::{Error, ResponseOk},
};

use crate::components::layouts::product::Index as Product;
use crate::templates::{components::NavLink, *};

use crate::models;
use crate::templates;
use crate::utils::tabs;
use crate::utils::urls;
use std::collections::HashMap;

// Returns snapshots page
#[get("/snapshots")]
pub async fn snapshots(cluster: &Cluster, _connected: ConnectedCluster<'_>) -> Result<ResponseOk, Error> {
    let mut layout = Product::new("Dashboard", &cluster);
    layout.breadcrumbs(vec![NavLink::new("Snapshots", &urls::deployment_snapshots()).active()]);

    let tabs = vec![tabs::Tab {
        name: "Snapshots",
        content: SnapshotsTab {}.render_once().unwrap(),
    }];

    let nav_tabs = tabs::Tabs::new(tabs, Some("Snapshots"), Some("Snapshots"))?;

    Ok(ResponseOk(layout.render(templates::Dashboard::new(nav_tabs))))
}

// Returns the specific snapshot page
#[get("/snapshots/<snapshot_id>")]
pub async fn snapshot(
    cluster: &Cluster,
    snapshot_id: i64,
    _connected: ConnectedCluster<'_>,
) -> Result<ResponseOk, Error> {
    let snapshot = models::Snapshot::get_by_id(cluster.pool(), snapshot_id).await?;

    let mut layout = Product::new("Dashboard", &cluster);
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
#[get("/snapshots_turboframe")]
pub async fn snapshots_index(cluster: ConnectedCluster<'_>) -> Result<ResponseOk, Error> {
    let snapshots = models::Snapshot::all(cluster.pool()).await?;

    Ok(ResponseOk(templates::Snapshots { snapshots }.render_once().unwrap()))
}

// Returns a specific snapshot for the deployment in a turboframe.
#[get("/snapshots_turboframe/<id>")]
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

pub fn routes() -> Vec<Route> {
    routes![snapshots, snapshot, snapshots_index, snapshots_get,]
}
