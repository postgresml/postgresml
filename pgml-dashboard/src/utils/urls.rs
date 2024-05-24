// Url to the deployments notebooks page.
pub fn deployment_notebooks() -> String {
    "/engine/notebooks".to_string()
}

// Url to a deployments specific notebook page.
pub fn deployment_notebook_by_id(notebook_id: i64) -> String {
    format!("/engine/notebooks/{}", notebook_id)
}

// Root of notebooks turboframes.
pub fn deployment_notebooks_turboframe() -> String {
    "/engine/notebooks_turboframe".to_string()
}

// Url to the deployments projects page.
pub fn deployment_projects() -> String {
    "/engine/projects".to_string()
}

// Url to a deployments specific project page.
pub fn deployment_project_by_id(project_id: i64) -> String {
    format!("/engine/projects/{}", project_id)
}

// Root of projects turboframes.
pub fn deployment_projects_turboframe() -> String {
    "/engine/projects_turboframe".to_string()
}

// Url to the deployments models page.
pub fn deployment_models() -> String {
    "/engine/models".to_string()
}

// Url to a deployments specific model page.
pub fn deployment_model_by_id(model_id: i64) -> String {
    format!("/engine/models/{}", model_id)
}

// Root of models turboframes.
pub fn deployment_models_turboframe() -> String {
    "/engine/models_turboframe".to_string()
}

// Url to the deployments snapshots page.
pub fn deployment_snapshots() -> String {
    "/engine/snapshots".to_string()
}

// Url to a deployments specific snapshot page.
pub fn deployment_snapshot_by_id(snapshot_id: i64) -> String {
    format!("/engine/snapshots/{}", snapshot_id)
}

// Root of snapshots turboframes.
pub fn deployment_snapshots_turboframe() -> String {
    "/engine/snapshots_turboframe".to_string()
}

// Url to the deployments uploader page.
pub fn deployment_uploader() -> String {
    "/engine/uploader".to_string()
}

// Root of uploader turboframes.
pub fn deployment_uploader_turboframe() -> String {
    "/engine/uploader_turboframe".to_string()
}
