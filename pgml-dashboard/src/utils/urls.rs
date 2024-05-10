// Url to the deployments notebooks page.
pub fn deployment_notebooks() -> String {
    // "/dashboard?tab=Notebook".to_string()
    "/dashboard/notebooks".to_string()
}

// Url to a deployments specific notebook page.
pub fn deployment_notebook_by_id(notebook_id: i64) -> String {
    // "/dashboard?tab=Notebook&id=id".to_string()
    format!("/dashboard/notebooks/{}", notebook_id)
}

// Url to the deployments projects page.
pub fn deployment_projects() -> String {
    // "/dashboard?tab=Projects".to_string()
    "/dashboard/projects".to_string()
}

// Url to a deployments specific project page.
pub fn deployment_project_by_id(project_id: i64) -> String {
    // "/dashboard?tab=Projects&id=id".to_string()
    format!("/dashboard/projects/{}", project_id)
}

// Url to the deployments models page.
pub fn deployment_models() -> String {
    // "/dashboard?tab=Models".to_string()
    "/dashboard/models".to_string()
}

// Url to a deployments specific model page.
pub fn deployment_model_by_id(model_id: i64) -> String {
    // "/dashboard?tab=Models&id=id".to_string()
    format!("/dashboard/models/{}", model_id)
}

// Url to the deployments snapshots page.
pub fn deployment_snapshots() -> String {
    // "/dashboard?tab=Snapshots".to_string()
    "/dashboard/snapshots".to_string()
}

// Url to a deployments specific snapshot page.
pub fn deployment_snapshot_by_id(snapshot_id: i64) -> String {
    // "/dashboard?tab=Snapshots&id=id".to_string()
    format!("/dashboard/snapshots/{}", snapshot_id)
}

// Url to the deployments uploader page.
pub fn deployment_uploader() -> String {
    // "/dashboard?tab=Upload_Data".to_string()
    "/dashboard/uploader".to_string()
}
