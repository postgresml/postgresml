use sailfish::TemplateOnce;

use crate::{
    guards::ConnectedCluster,
    responses::{Error, ResponseOk},
};

use crate::models;
use crate::templates;

use std::collections::HashMap;

pub mod deployment_models;
pub mod notebooks;
pub mod projects;
pub mod snapshots;
pub mod uploader;

// #[get("/deployments")]
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
        templates::Deployments { projects, deployments }.render_once().unwrap(),
    ))
}

// #[get("/deployments/<id>")]
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

pub fn routes() -> Vec<Route> {
    let mut routes = routes![deployments_index, deployments_get,];

    routes.extend(deployment_models::routes());
    routes.extend(notebooks::routes());
    routes.extend(projects::routes());
    routes.extend(snapshots::routes());
    routes.extend(uploader::routes());
    routes
}
