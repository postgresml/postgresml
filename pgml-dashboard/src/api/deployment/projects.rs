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

pub fn routes() -> Router {
    Router::new()
        .route("/projects", get(projects))
        .route("/projects/:project_id", get(project))
        .route("/projects_turboframe", get(project_index))
        .route("/projects_turboframe/:id", get(project_get))
}

// Returns the deployments projects page.
pub async fn projects(
    Extension(cluster): Extension<Cluster>,
    _connected: ConnectedCluster,
) -> Result<ResponseOk, Error> {
    let mut layout = crate::templates::WebAppBase::new("Dashboard", &cluster);
    layout.breadcrumbs(vec![NavLink::new("Projects", &urls::deployment_projects()).active()]);

    let tabs = vec![tabs::Tab {
        name: "Projects",
        content: ProjectsTab {}.render_once().unwrap(),
    }];

    let nav_tabs = tabs::Tabs::new(tabs, Some("Notebooks"), Some("Projects"))?;

    Ok(ResponseOk(layout.render(templates::Dashboard::new(nav_tabs))))
}

// Return the specified project page.
pub async fn project(
    Extension(cluster): Extension<Cluster>,
    Path(project_id): Path<i64>,
    _connected: ConnectedCluster,
) -> Result<ResponseOk, Error> {
    let project = models::Project::get_by_id(cluster.pool(), project_id).await?;

    let mut layout = crate::templates::WebAppBase::new("Dashboard", &cluster);
    layout.breadcrumbs(vec![
        NavLink::new("Projects", &urls::deployment_projects()),
        NavLink::new(project.name.as_str(), &urls::deployment_project_by_id(project_id)).active(),
    ]);

    let tabs = vec![tabs::Tab {
        name: "Project",
        content: ProjectTab { project_id }.render_once().unwrap(),
    }];

    let nav_tabs = tabs::Tabs::new(tabs, Some("Projects"), Some("Projects"))?;

    Ok(ResponseOk(layout.render(templates::Dashboard::new(nav_tabs))))
}

// Returns all the deployments for the project in a turbo frame.
pub async fn project_index(cluster: ConnectedCluster) -> Result<ResponseOk, Error> {
    Ok(ResponseOk(
        templates::Projects {
            projects: models::Project::all(cluster.pool()).await?,
        }
        .render_once()
        .unwrap(),
    ))
}

// Returns the specified project page.
pub async fn project_get(cluster: ConnectedCluster, Path(id): Path<i64>) -> Result<ResponseOk, Error> {
    let project = models::Project::get_by_id(cluster.pool(), id).await?;
    let models = models::Model::get_by_project_id(cluster.pool(), id).await?;

    Ok(ResponseOk(
        templates::Project { project, models }.render_once().unwrap(),
    ))
}
