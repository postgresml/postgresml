use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

use pgx::*;

use crate::orm::Snapshot;
use crate::orm::Task;

#[derive(Debug)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub task: Task,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Display for Project {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "Project {{ id: {}, name: {}, task: {:?} }}",
            self.id, self.name, self.task
        )
    }
}

impl Project {
    pub fn find(id: i64) -> Option<Project> {
        let mut project: Option<Project> = None;

        Spi::connect(|client| {
            let result = client.select("SELECT id, name, task::TEXT, created_at, updated_at FROM pgml.projects WHERE id = $1 LIMIT 1;",
                Some(1),
                Some(vec![
                    (PgBuiltInOids::INT8OID.oid(), id.into_datum()),
                ])
            ).first();
            if !result.is_empty() {
                project = Some(Project {
                    id: result.get_datum(1).unwrap(),
                    name: result.get_datum(2).unwrap(),
                    task: Task::from_str(result.get_datum(3).unwrap()).unwrap(),
                    created_at: result.get_datum(4).unwrap(),
                    updated_at: result.get_datum(5).unwrap(),
                });
            }
            Ok(Some(1))
        });

        project
    }

    pub fn find_by_name(name: &str) -> Option<Project> {
        let mut project = None;

        Spi::connect(|client| {
            let result = client.select("SELECT id, name, task::TEXT, created_at, updated_at FROM pgml.projects WHERE name = $1 LIMIT 1;",
                Some(1),
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), name.into_datum()),
                ])
            ).first();
            if !result.is_empty() {
                project = Some(Project {
                    id: result.get_datum(1).unwrap(),
                    name: result.get_datum(2).unwrap(),
                    task: Task::from_str(result.get_datum(3).unwrap()).unwrap(),
                    created_at: result.get_datum(4).unwrap(),
                    updated_at: result.get_datum(5).unwrap(),
                });
            }
            Ok(Some(1))
        });

        project
    }

    pub fn create(name: &str, task: Task) -> Project {
        let mut project: Option<Project> = None;

        Spi::connect(|client| {
            let result = client.select(r#"INSERT INTO pgml.projects (name, task) VALUES ($1, $2::pgml.task) RETURNING id, name, task::TEXT, created_at, updated_at;"#,
                Some(1),
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), name.into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), task.to_string().into_datum()),
                ])
            ).first();
            if !result.is_empty() {
                project = Some(Project {
                    id: result.get_datum(1).unwrap(),
                    name: result.get_datum(2).unwrap(),
                    task: Task::from_str(result.get_datum(3).unwrap()).unwrap(),
                    created_at: result.get_datum(4).unwrap(),
                    updated_at: result.get_datum(5).unwrap(),
                });
            }
            Ok(Some(1))
        });
        project.unwrap()
    }

    pub fn last_snapshot(&self) -> Option<Snapshot> {
        Snapshot::find_last_by_project_id(self.id)
    }
}
