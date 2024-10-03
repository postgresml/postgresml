use axum::{extract::Path, routing::get, Extension, Router};
use sailfish::TemplateOnce;

use crate::{
    guards::Cluster,
    guards::ConnectedCluster,
    responses::{Error, ResponseOk},
};

use crate::templates::{components::NavLink, *};

use crate::models;
use crate::templates;
use crate::utils::tabs;
use crate::utils::urls;

use std::collections::HashMap;

pub fn routes() -> Router {
    Router::new()
        .route("/models", get(deployment_models))
        .route("/models/:model_id", get(model))
        .route("/models_turboframe", get(models_index))
        .route("/models_turboframe/:id", get(models_get))
}

// Returns models page
pub async fn deployment_models(
    Extension(cluster): Extension<Cluster>,
    _connected: ConnectedCluster,
) -> Result<ResponseOk, Error> {
    let mut layout = crate::templates::WebAppBase::new("Dashboard", &cluster);
    layout.breadcrumbs(vec![NavLink::new("Models", &urls::deployment_models()).active()]);

    let tabs = vec![tabs::Tab {
        name: "Models",
        content: ModelsTab {}.render_once().unwrap(),
    }];

    let nav_tabs = tabs::Tabs::new(tabs, Some("Models"), Some("Models"))?;

    Ok(ResponseOk(layout.render(templates::Dashboard::new(nav_tabs))))
}

// Returns models page
pub async fn model(
    Extension(cluster): Extension<Cluster>,
    Path(model_id): Path<i64>,
    _connected: ConnectedCluster,
) -> Result<ResponseOk, Error> {
    let model = models::Model::get_by_id(cluster.pool(), model_id).await?;
    let project = models::Project::get_by_id(cluster.pool(), model.project_id).await?;

    let mut layout = crate::templates::WebAppBase::new("Dashboard", &cluster);
    layout.breadcrumbs(vec![
        NavLink::new("Models", &urls::deployment_models()),
        NavLink::new(&project.name, &urls::deployment_project_by_id(project.id)),
        NavLink::new(&model.algorithm, &urls::deployment_model_by_id(model.id)).active(),
    ]);

    let tabs = vec![tabs::Tab {
        name: "Model",
        content: ModelTab { model_id }.render_once().unwrap(),
    }];

    let nav_tabs = tabs::Tabs::new(tabs, Some("Models"), Some("Models"))?;

    Ok(ResponseOk(layout.render(templates::Dashboard::new(nav_tabs))))
}

pub async fn models_index(cluster: ConnectedCluster) -> Result<ResponseOk, Error> {
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

pub async fn models_get(cluster: ConnectedCluster, Path(id): Path<i64>) -> Result<ResponseOk, Error> {
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
