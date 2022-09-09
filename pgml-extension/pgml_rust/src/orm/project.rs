use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use pgx::*;

use crate::orm::Model;
use crate::orm::Snapshot;
use crate::orm::Task;

static PROJECTS_BY_NAME: Lazy<Mutex<HashMap<String, Arc<Project>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub task: Task,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    deployed_model: Option<Model>,
}

impl Project {
    pub fn find(id: i64) -> Option<Project> {
        let mut project: Option<Project> = None;

        Spi::connect(|client| {
            let result = client.select("SELECT id, name, task, created_at, updated_at FROM pgml_rust.projects WHERE id = $1 LIMIT 1;",
                Some(1),
                Some(vec![
                    (PgBuiltInOids::INT8OID.oid(), id.into_datum()),
                ])
            ).first();
            if result.len() > 0 {
                project = Some(Project {
                    id: result.get_datum(1).unwrap(),
                    name: result.get_datum(2).unwrap(),
                    task: Task::from_str(result.get_datum(3).unwrap()).unwrap(),
                    created_at: result.get_datum(4).unwrap(),
                    updated_at: result.get_datum(5).unwrap(),
                    deployed_model: None,
                });
            }
            Ok(Some(1))
        });

        project
    }

    pub fn find_by_name(name: &str) -> Option<Arc<Project>> {
        {
            let projects = PROJECTS_BY_NAME.lock().unwrap();
            let project = projects.get(name);
            if project.is_some() {
                info!("cache hit: {}", name);
                return Some(project.unwrap().clone());
            } else {
                info!("cache miss: {}", name);
            }
        }

        let mut project = None;

        Spi::connect(|client| {
            let result = client.select("SELECT id, name, task, created_at, updated_at FROM pgml_rust.projects WHERE name = $1 LIMIT 1;",
                Some(1),
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), name.into_datum()),
                ])
            ).first();
            if result.len() > 0 {
                info!("db hit: {}", name);
                let mut projects = PROJECTS_BY_NAME.lock().unwrap();
                projects.insert(
                    name.to_string(),
                    Arc::new(Project {
                        id: result.get_datum(1).unwrap(),
                        name: result.get_datum(2).unwrap(),
                        task: Task::from_str(result.get_datum(3).unwrap()).unwrap(),
                        created_at: result.get_datum(4).unwrap(),
                        updated_at: result.get_datum(5).unwrap(),
                        deployed_model: None,
                    }),
                );
                project = Some(projects.get(name).unwrap().clone());
            } else {
                info!("db miss: {}", name);
            }
            Ok(Some(1))
        });

        project
    }

    pub fn create(name: &str, task: Task) -> Arc<Project> {
        let mut project: Option<Arc<Project>> = None;

        Spi::connect(|client| {
            let result = client.select(r#"INSERT INTO pgml_rust.projects (name, task) VALUES ($1, $2::pgml_rust.task) RETURNING id, name, task, created_at, updated_at;"#,
                Some(1),
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), name.into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), task.to_string().into_datum()),
                ])
            ).first();
            if result.len() > 0 {
                let mut projects = PROJECTS_BY_NAME.lock().unwrap();
                projects.insert(
                    name.to_string(),
                    Arc::new(Project {
                        id: result.get_datum(1).unwrap(),
                        name: result.get_datum(2).unwrap(),
                        task: result.get_datum(3).unwrap(),
                        created_at: result.get_datum(4).unwrap(),
                        updated_at: result.get_datum(5).unwrap(),
                        deployed_model: None,
                    }),
                );
                project = Some(projects.get(name).unwrap().clone());
            }
            Ok(Some(1))
        });
        info!("create project: {:?}", project.as_ref().unwrap());
        project.unwrap()
    }

    pub fn last_snapshot(&self) -> Option<Snapshot> {
        Snapshot::find_last_by_project_id(self.id)
    }
}
