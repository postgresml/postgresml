use axum::{extract::Path, routing::get};
use sailfish::TemplateOnce;

use crate::{
    guards::ConnectedCluster,
    responses::{Error, ResponseOk},
    Router,
};

use crate::models;
use crate::templates;

use std::collections::HashMap;

pub mod deployment_models;
pub mod notebooks;
pub mod projects;
pub mod snapshots;
pub mod uploader;

pub fn routes() -> Router {
    Router::new()
        .route("/deployments", get(deployments_index))
        .route("/deployments/:id", get(deployments_get))
        .merge(deployment_models::routes())
        .merge(notebooks::routes())
        .merge(projects::routes())
        .merge(snapshots::routes())
        .merge(uploader::routes())
}

pub async fn deployments_index(cluster: ConnectedCluster) -> Result<ResponseOk, Error> {
    let projects = models::Project::all(cluster.pool()).await?;
    let mut deployments = HashMap::new();

    for project in projects.iter() {
        deployments.insert(
            project.id,
            models::Deployment::get_by_project_id(cluster.pool(), project.id).await?,
        );
    }

    Ok(ResponseOk(
        templates::Deployments { projects, deployments }.render_once().unwrap(),
    ))
}

pub async fn deployments_get(cluster: ConnectedCluster, Path(id): Path<i64>) -> Result<ResponseOk, Error> {
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
