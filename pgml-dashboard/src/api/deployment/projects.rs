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

// Returns the deployments projects page.
#[get("/projects")]
pub async fn projects(cluster: &Cluster, _connected: ConnectedCluster<'_>) -> Result<ResponseOk, Error> {
    let mut layout = Product::new("Dashboard", &cluster);
    layout.breadcrumbs(vec![NavLink::new("Projects", &urls::deployment_projects()).active()]);

    let tabs = vec![tabs::Tab {
        name: "Projects",
        content: ProjectsTab {}.render_once().unwrap(),
    }];

    let nav_tabs = tabs::Tabs::new(tabs, Some("Notebooks"), Some("Projects"))?;

    Ok(ResponseOk(layout.render(templates::Dashboard::new(nav_tabs))))
}

// Return the specified project page.
#[get("/projects/<project_id>")]
pub async fn project(
    cluster: &Cluster,
    project_id: i64,
    _connected: ConnectedCluster<'_>,
) -> Result<ResponseOk, Error> {
    let project = models::Project::get_by_id(cluster.pool(), project_id).await?;

    let mut layout = Product::new("Dashboard", &cluster);
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
#[get("/projects_turboframe")]
pub async fn project_index(cluster: ConnectedCluster<'_>) -> Result<ResponseOk, Error> {
    Ok(ResponseOk(
        templates::Projects {
            projects: models::Project::all(cluster.pool()).await?,
        }
        .render_once()
        .unwrap(),
    ))
}

// Returns the specified project page.
#[get("/projects_turboframe/<id>")]
pub async fn project_get(cluster: ConnectedCluster<'_>, id: i64) -> Result<ResponseOk, Error> {
    let project = models::Project::get_by_id(cluster.pool(), id).await?;
    let models = models::Model::get_by_project_id(cluster.pool(), id).await?;

    Ok(ResponseOk(
        templates::Project { project, models }.render_once().unwrap(),
    ))
}

pub fn routes() -> Vec<Route> {
    routes![projects, project, project_index, project_get,]
}
