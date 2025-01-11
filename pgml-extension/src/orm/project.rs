use parking_lot::Mutex;
use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

use hash32::{BuildHasherDefault, FnvHasher};
use heapless::IndexMap;
use once_cell::sync::Lazy;
use pgrx::{datum::*, *}; // Use FnvHasher directly instead of dyn Hasher

use crate::orm::*;

// We need a wrapper to implement PGRXSharedMemory for IndexMap
#[derive(Default)]
pub struct ProjectIdMap(IndexMap<i64, i64, BuildHasherDefault<FnvHasher>, 1024>);

unsafe impl PGRXSharedMemory for ProjectIdMap {}

impl ProjectIdMap {
    pub fn new() -> Self {
        Self(IndexMap::new())
    }

    pub fn insert(&mut self, project_id: i64, model_id: i64) -> Option<i64> {
        self.0.insert(project_id, model_id).unwrap()
    }

    pub fn get(&self, project_id: &i64) -> Option<i64> {
        self.0.get(project_id).copied()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

// Wrapper for the PgLwLock
pub struct ProjectDeploymentMap(PgLwLock<ProjectIdMap>);

impl ProjectDeploymentMap {
    pub const fn new() -> Self {
        Self(PgLwLock::new())
    }

    pub fn insert(&'static self, project_id: i64, model_id: i64) -> Option<i64> {
        self.0.exclusive().insert(project_id, model_id)
    }

    pub fn get(&'static self, project_id: &i64) -> Option<i64> {
        self.0.share().get(project_id)
    }

    pub fn clear(&'static self) {
        self.0.exclusive().clear()
    }

    pub fn len(&'static self) -> usize {
        self.0.share().len()
    }

    pub fn lock(&'static self) -> &'static PgLwLock<ProjectIdMap> {
        &self.0
    }
}

// Implement the required traits for our wrapper
unsafe impl PGRXSharedMemory for ProjectDeploymentMap {}

impl PgSharedMemoryInitialization for ProjectDeploymentMap {
    fn pg_init(&'static self) {
        PgSharedMem::pg_init_locked(&self.0);
    }

    unsafe fn shmem_init(&'static self) {
        unsafe {
            PgSharedMem::shmem_init_locked(&self.0);
        }
    }
}

// Static declaration
static PROJECT_ID_TO_DEPLOYED_MODEL_ID: ProjectDeploymentMap = ProjectDeploymentMap::new();

static PROJECT_NAME_TO_PROJECT_ID: Lazy<Mutex<HashMap<String, i64>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Initialize shared memory.
/// # Note
/// Only call from `_PG_init`.
pub fn init() {
    pg_shmem_init!(PROJECT_ID_TO_DEPLOYED_MODEL_ID);
}

#[derive(Debug, Clone)]
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
    pub fn get_deployed_model_id(project_name: &str) -> i64 {
        let mut projects = PROJECT_NAME_TO_PROJECT_ID.lock();
        let project_id = match projects.get(project_name) {
            Some(project_id) => *project_id,
            None => {
                let result = Spi::get_two_with_args::<i64, i64>(
                    "SELECT deployments.project_id, deployments.model_id 
                    FROM pgml.deployments
                    JOIN pgml.projects ON projects.id = deployments.project_id
                    WHERE projects.name = $1 
                    ORDER BY deployments.created_at DESC
                    LIMIT 1",
                    vec![(PgBuiltInOids::TEXTOID.oid(), project_name.into_datum())],
                );
                let (project_id, model_id) = match result {
                    Ok(o) => o,
                    Err(_) => error!("No deployed model exists for the project named: `{}`", project_name),
                };
                let project_id = project_id
                    .unwrap_or_else(|| error!("No deployed model exists for the project named: `{}`", project_name));
                let model_id = model_id
                    .unwrap_or_else(|| error!("No deployed model exists for the project named: `{}`", project_name));
                projects.insert(project_name.to_string(), project_id);
                let mut projects = PROJECT_ID_TO_DEPLOYED_MODEL_ID.0.exclusive();
                if projects.len() == 1024 {
                    warning!("Active projects have exceeded capacity map, clearing caches.");
                    projects.clear();
                }
                projects.insert(project_id, model_id).unwrap();
                project_id
            }
        };
        PROJECT_ID_TO_DEPLOYED_MODEL_ID.0.share().get(&project_id).unwrap()
    }

    pub fn deploy(&self, model_id: i64, strategy: Strategy) {
        info!("Deploying model id: {:?}", model_id);
        Spi::get_one_with_args::<i64>(
            "INSERT INTO pgml.deployments (project_id, model_id, strategy) VALUES ($1, $2, $3::pgml.strategy) RETURNING id",
            vec![
                (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
                (PgBuiltInOids::INT8OID.oid(), model_id.into_datum()),
                (PgBuiltInOids::TEXTOID.oid(), strategy.to_string().into_datum()),
            ],
        ).expect("Deployment to be insertable");
        let mut projects = PROJECT_ID_TO_DEPLOYED_MODEL_ID.0.exclusive();
        if projects.len() == 1024 {
            warning!("Active projects has exceeded capacity map, clearing caches.");
            projects.clear();
        }
        projects.insert(self.id, model_id);
    }

    pub fn find(id: i64) -> Option<Project> {
        let mut project: Option<Project> = None;

        Spi::connect(|client| {
            let result = client
                .select(
                    "SELECT id, name, task::TEXT, created_at, updated_at FROM pgml.projects WHERE id = $1 LIMIT 1;",
                    Some(1),
                    Some(vec![(PgBuiltInOids::INT8OID.oid(), id.into_datum())]),
                )
                .unwrap()
                .first();
            if !result.is_empty() {
                project = Some(Project {
                    id: result.get(1).unwrap().unwrap(),
                    name: result.get(2).unwrap().unwrap(),
                    task: Task::from_str(result.get(3).unwrap().unwrap()).unwrap(),
                    created_at: result.get(4).unwrap().unwrap(),
                    updated_at: result.get(5).unwrap().unwrap(),
                });
            }
        });

        project
    }

    pub fn find_by_name(name: &str) -> Option<Project> {
        let mut project = None;

        Spi::connect(|client| {
            let result = client
                .select(
                    "SELECT id, name, task::TEXT, created_at, updated_at FROM pgml.projects WHERE name = $1 LIMIT 1;",
                    Some(1),
                    Some(vec![(PgBuiltInOids::TEXTOID.oid(), name.into_datum())]),
                )
                .unwrap()
                .first();
            if !result.is_empty() {
                project = Some(Project {
                    id: result.get(1).unwrap().unwrap(),
                    name: result.get(2).unwrap().unwrap(),
                    task: Task::from_str(result.get(3).unwrap().unwrap()).unwrap(),
                    created_at: result.get(4).unwrap().unwrap(),
                    updated_at: result.get(5).unwrap().unwrap(),
                });
            }
        });

        project
    }

    pub fn create(name: &str, task: Task) -> Project {
        let mut project: Option<Project> = None;

        Spi::connect(|mut client| {
            let result = client.update(r#"INSERT INTO pgml.projects (name, task) VALUES ($1, $2::pgml.task) RETURNING id, name, task::TEXT, created_at, updated_at;"#,
                Some(1),
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), name.into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), task.to_pg_enum().into_datum()),
                ])
            ).unwrap().first();
            if !result.is_empty() {
                project = Some(Project {
                    id: result.get(1).unwrap().unwrap(),
                    name: result.get(2).unwrap().unwrap(),
                    task: Task::from_str(result.get(3).unwrap().unwrap()).unwrap(),
                    created_at: result.get(4).unwrap().unwrap(),
                    updated_at: result.get(5).unwrap().unwrap(),
                });
            }
        });
        project.unwrap()
    }

    pub fn last_snapshot(&self) -> Option<Snapshot> {
        Snapshot::find_last_by_project_id(self.id)
    }
}
