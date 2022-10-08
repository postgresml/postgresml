use parking_lot::Mutex;
use std::collections::HashMap;
use std::fmt::Write;
use std::str::FromStr;

use once_cell::sync::Lazy;
use pgx::*;
#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde_json::json;

use crate::orm::Algorithm;
use crate::orm::Model;
use crate::orm::Project;
use crate::orm::Runtime;
use crate::orm::Sampling;
use crate::orm::Search;
use crate::orm::Snapshot;
use crate::orm::Strategy;
use crate::orm::Task;

static PROJECT_ID_TO_DEPLOYED_MODEL_ID: PgLwLock<heapless::FnvIndexMap<i64, i64, 1024>> =
    PgLwLock::new();
static PROJECT_NAME_TO_PROJECT_ID: Lazy<Mutex<HashMap<String, i64>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[pg_guard]
pub extern "C" fn _PG_init() {
    pg_shmem_init!(PROJECT_ID_TO_DEPLOYED_MODEL_ID);
}

#[cfg(feature = "python")]
#[pg_extern]
pub fn validate_python_dependencies() {
    Python::with_gil(|py| {
        let sys = PyModule::import(py, "sys").unwrap();
        let version: String = sys.getattr("version").unwrap().extract().unwrap();
        info!("Python version: {version}");
        for module in ["xgboost", "lightgbm", "numpy", "sklearn"] {
            match py.import(module) {
                Ok(_) => (),
                Err(e) => {
                    panic!(
                        "The {module} package is missing. Install it with `sudo pip3 install {module}`\n{e}"
                    );
                }
            }
        }
    });

    let sklearn_version = sklearn_version();

    info!(
        "Scikit-learn {}, XGBoost 1.62, LightGBM 3.3.2",
        sklearn_version
    );
}

#[cfg(not(feature = "python"))]
#[pg_extern]
pub fn validate_python_dependencies() {}

#[cfg(feature = "python")]
#[pg_extern]
pub fn sklearn_version() -> String {
    let mut version = String::new();

    Python::with_gil(|py| {
        let sklearn = py.import("sklearn").unwrap();
        version = sklearn.getattr("__version__").unwrap().extract().unwrap();
    });

    version
}

#[cfg(not(feature = "python"))]
#[pg_extern]
pub fn sklearn_version() -> String {
    String::from("Scikit-learn is not installed, recompile with `--features python`")
}

#[cfg(feature = "python")]
#[pg_extern]
fn python_version() -> String {
    let mut version = String::new();

    Python::with_gil(|py| {
        let sys = PyModule::import(py, "sys").unwrap();
        version = sys.getattr("version").unwrap().extract().unwrap();
    });

    version
}

#[cfg(not(feature = "python"))]
#[pg_extern]
pub fn python_version() -> String {
    String::from("Python is not installed, recompile with `--features python`")
}

#[pg_extern]
fn version() -> String {
    crate::VERSION.to_string()
}

#[allow(clippy::too_many_arguments)]
#[pg_extern]
fn train(
    project_name: &str,
    task: Option<default!(Task, "NULL")>,
    relation_name: Option<default!(&str, "NULL")>,
    y_column_name: Option<default!(&str, "NULL")>,
    algorithm: default!(Algorithm, "'linear'"),
    hyperparams: default!(JsonB, "'{}'"),
    search: Option<default!(Search, "NULL")>,
    search_params: default!(JsonB, "'{}'"),
    search_args: default!(JsonB, "'{}'"),
    test_size: default!(f32, 0.25),
    test_sampling: default!(Sampling, "'last'"),
    runtime: Option<default!(Runtime, "NULL")>,
    automatic_deploy: Option<default!(bool, true)>,
) -> impl std::iter::Iterator<
    Item = (
        name!(project, String),
        name!(task, String),
        name!(algorithm, String),
        name!(deployed, bool),
    ),
> {
    let project = match Project::find_by_name(project_name) {
        Some(project) => project,
        None => Project::create(project_name, match task {
            Some(task) => task,
            None => error!("Project `{}` does not exist. To create a new project, provide the task (regression or classification).", project_name),
        }),
    };

    if task.is_some() && task.unwrap() != project.task {
        error!("Project `{:?}` already exists with a different task: `{:?}`. Create a new project instead.", project.name, project.task);
    }

    let snapshot = match relation_name {
        None => {
            let snapshot = project
                .last_snapshot()
                .expect("You must pass a `relation_name` and `y_column_name` to snapshot the first time you train a model.");

            info!("Using existing snashot from {}", snapshot.snapshot_name(),);

            snapshot
        }

        Some(relation_name) => {
            info!(
                "Snapshotting table \"{}\", this may take a little while...",
                relation_name
            );

            let snapshot = Snapshot::create(
                relation_name,
                y_column_name
                    .expect("You must pass a `y_column_name` when you pass a `relation_name`"),
                test_size,
                test_sampling,
            );

            info!(
                "Snapshot of table \"{}\" created and saved in {}",
                relation_name,
                snapshot.snapshot_name(),
            );

            snapshot
        }
    };

    // # Default repeatable random state when possible
    // let algorithm = Model.algorithm_from_name_and_task(algorithm, task);
    // if "random_state" in algorithm().get_params() and "random_state" not in hyperparams:
    //     hyperparams["random_state"] = 0
    let model = Model::create(
        &project,
        &snapshot,
        algorithm,
        hyperparams,
        search,
        search_params,
        search_args,
        runtime,
    );

    let new_metrics: &serde_json::Value = &model.metrics.unwrap().0;
    let new_metrics = new_metrics.as_object().unwrap();

    let deployed_metrics = Spi::get_one_with_args::<JsonB>(
        "
        SELECT models.metrics
        FROM pgml.models
        JOIN pgml.deployments 
            ON deployments.model_id = models.id
        JOIN pgml.projects
            ON projects.id = deployments.project_id
        WHERE projects.name = $1
        ORDER by deployments.created_at DESC
        LIMIT 1;",
        vec![(PgBuiltInOids::TEXTOID.oid(), project_name.into_datum())],
    );

    let mut deploy = true;

    match automatic_deploy {
        // Deploy only if metrics are better than previous model.
        Some(true) | None => {
            if let Some(deployed_metrics) = deployed_metrics {
                let deployed_metrics = deployed_metrics.0.as_object().unwrap();
                match project.task {
                    Task::classification => {
                        if deployed_metrics.get("f1").unwrap().as_f64()
                            > new_metrics.get("f1").unwrap().as_f64()
                        {
                            deploy = false;
                        }
                    }
                    Task::regression => {
                        if deployed_metrics.get("r2").unwrap().as_f64()
                            > new_metrics.get("r2").unwrap().as_f64()
                        {
                            deploy = false;
                        }
                    }
                }
            }
        }

        Some(false) => deploy = false,
    };

    if deploy {
        Spi::get_one_with_args::<i64>(
            "INSERT INTO pgml.deployments (project_id, model_id, strategy) VALUES ($1, $2, $3::pgml.strategy) RETURNING id",
            vec![
                (PgBuiltInOids::INT8OID.oid(), project.id.into_datum()),
                (PgBuiltInOids::INT8OID.oid(), model.id.into_datum()),
                (PgBuiltInOids::TEXTOID.oid(), Strategy::most_recent.to_string().into_datum()),
            ],
        );
        let mut projects = PROJECT_ID_TO_DEPLOYED_MODEL_ID.exclusive();
        if projects.len() == 1024 {
            warning!("Active projects has exceeded capacity map, clearing caches.");
            projects.clear();
        }
        projects.insert(project.id, model.id).unwrap();
    }

    vec![(
        project.name,
        project.task.to_string(),
        model.algorithm.to_string(),
        deploy,
    )]
    .into_iter()
}

#[pg_extern]
fn deploy(
    project_name: &str,
    strategy: Strategy,
    algorithm: Option<default!(Algorithm, "NULL")>,
) -> impl std::iter::Iterator<
    Item = (
        name!(project, String),
        name!(strategy, String),
        name!(algorithm, String),
    ),
> {
    let (project_id, task) = Spi::get_two_with_args::<i64, String>(
        "SELECT id, task::TEXT from pgml.projects WHERE name = $1",
        vec![(PgBuiltInOids::TEXTOID.oid(), project_name.into_datum())],
    );

    let project_id =
        project_id.unwrap_or_else(|| error!("Project named `{}` does not exist.", project_name));

    let task = Task::from_str(&task.unwrap()).unwrap();

    let mut sql = "SELECT models.id, models.algorithm::TEXT FROM pgml.models JOIN pgml.projects ON projects.id = models.project_id".to_string();
    let mut predicate = "\nWHERE projects.name = $1".to_string();
    if let Some(algorithm) = algorithm {
        let _ = write!(
            predicate,
            "\nAND algorithm::TEXT = '{}'",
            algorithm.to_string().as_str()
        );
    }
    match strategy {
        Strategy::best_score => match task {
            Task::regression => {
                let _ = write!(
                    sql,
                    "{predicate}\nORDER BY models.metrics->>'r2' DESC NULLS LAST"
                );
            }

            Task::classification => {
                let _ = write!(
                    sql,
                    "{predicate}\nORDER BY models.metrics->>'f1' DESC NULLS LAST"
                );
            }
        },

        Strategy::most_recent => {
            let _ = write!(sql, "{predicate}\nORDER by models.created_at DESC");
        }

        Strategy::rollback => {
            let _ = write!(
                sql,
                "
                JOIN pgml.deployments ON deployments.project_id = projects.id
                    AND deployments.model_id = models.id
                    AND models.id != (
                        SELECT deployments.model_id
                        FROM pgml.deployments 
                        JOIN pgml.projects
                            ON projects.id = deployments.project_id
                        WHERE projects.name = $1
                        ORDER by deployments.created_at DESC
                        LIMIT 1
                    )
                {predicate}
                ORDER by deployments.created_at DESC
            "
            );
        }
        _ => error!("invalid stategy"),
    }
    sql += "\nLIMIT 1";
    let (model_id, algorithm) = Spi::get_two_with_args::<i64, String>(
        &sql,
        vec![(PgBuiltInOids::TEXTOID.oid(), project_name.into_datum())],
    );
    let model_id = model_id.expect("No qualified models exist for this deployment.");
    let algorithm = algorithm.expect("No qualified models exist for this deployment.");

    Spi::get_one_with_args::<i64>(
        "INSERT INTO pgml.deployments (project_id, model_id, strategy) VALUES ($1, $2, $3::pgml.strategy) RETURNING id",
        vec![
            (PgBuiltInOids::INT8OID.oid(), project_id.into_datum()),
            (PgBuiltInOids::INT8OID.oid(), model_id.into_datum()),
            (PgBuiltInOids::TEXTOID.oid(), strategy.to_string().into_datum()),
        ]
    );

    let mut projects = PROJECT_ID_TO_DEPLOYED_MODEL_ID.exclusive();
    if projects.len() == 1024 {
        warning!("Active projects has exceeded capacity map, clearing caches.");
        projects.clear();
    }
    projects.insert(project_id, model_id).unwrap();

    vec![(project_name.to_string(), strategy.to_string(), algorithm)].into_iter()
}

#[pg_extern]
fn predict(project_name: &str, features: Vec<f32>) -> f32 {
    let mut projects = PROJECT_NAME_TO_PROJECT_ID.lock();
    let project_id = match projects.get(project_name) {
        Some(project_id) => *project_id,
        None => {
            let (project_id, model_id) = Spi::get_two_with_args::<i64, i64>(
                "SELECT deployments.project_id, deployments.model_id 
                FROM pgml.deployments
                JOIN pgml.projects ON projects.id = deployments.project_id
                WHERE projects.name = $1 
                ORDER BY deployments.created_at DESC
                LIMIT 1",
                vec![(PgBuiltInOids::TEXTOID.oid(), project_name.into_datum())],
            );
            let project_id = project_id.unwrap_or_else(|| {
                error!(
                    "No deployed model exists for the project named: `{}`",
                    project_name
                )
            });
            let model_id = model_id.unwrap_or_else(|| {
                error!(
                    "No deployed model exists for the project named: `{}`",
                    project_name
                )
            });
            projects.insert(project_name.to_string(), project_id);
            let mut projects = PROJECT_ID_TO_DEPLOYED_MODEL_ID.exclusive();
            if projects.len() == 1024 {
                warning!("Active projects have exceeded capacity map, clearing caches.");
                projects.clear();
            }
            projects.insert(project_id, model_id).unwrap();
            project_id
        }
    };

    let model_id = *PROJECT_ID_TO_DEPLOYED_MODEL_ID
        .share()
        .get(&project_id)
        .unwrap();
    let estimator = crate::orm::file::find_deployed_estimator_by_model_id(model_id);
    estimator.predict(&features)
}

#[pg_extern]
fn snapshot(
    relation_name: &str,
    y_column_name: &str,
    test_size: default!(f32, 0.25),
    test_sampling: default!(Sampling, "'last'"),
) -> impl std::iter::Iterator<Item = (name!(relation, String), name!(y_column_name, String))> {
    Snapshot::create(relation_name, y_column_name, test_size, test_sampling);
    vec![(relation_name.to_string(), y_column_name.to_string())].into_iter()
}

#[pg_extern]
fn load_dataset(
    source: &str,
    limit: Option<default!(i64, "NULL")>,
) -> impl std::iter::Iterator<Item = (name!(table_name, String), name!(rows, i64))> {
    // cast limit since pgx doesn't support usize
    let limit: Option<usize> = limit.map(|limit| limit.try_into().unwrap());
    let (name, rows) = match source {
        "breast_cancer" => crate::orm::dataset::load_breast_cancer(limit),
        "diabetes" => crate::orm::dataset::load_diabetes(limit),
        "digits" => crate::orm::dataset::load_digits(limit),
        "iris" => crate::orm::dataset::load_iris(limit),
        _ => error!("Unknown source: `{source}`"),
    };

    vec![(name, rows)].into_iter()
}

#[pg_extern]
fn model_predict(model_id: i64, features: Vec<f32>) -> f32 {
    let estimator = crate::orm::file::find_deployed_estimator_by_model_id(model_id);
    estimator.predict(&features)
}

#[pg_extern]
fn model_predict_batch(model_id: i64, features: Vec<f32>) -> Vec<f32> {
    let estimator = crate::orm::file::find_deployed_estimator_by_model_id(model_id);
    estimator.predict_batch(&features)
}

#[cfg(feature = "python")]
#[pg_extern(name = "transform")]
pub fn transform_json(
    task: JsonB,
    args: default!(JsonB, "'{}'"),
    inputs: default!(Vec<String>, "ARRAY[]::TEXT[]"),
) -> JsonB {
    JsonB(crate::bindings::transformers::transform(
        &task.0, &args.0, &inputs,
    ))
}

#[cfg(feature = "python")]
#[pg_extern(name = "transform")]
pub fn transform_string(
    task: String,
    args: default!(JsonB, "'{}'"),
    inputs: default!(Vec<String>, "ARRAY[]::TEXT[]"),
) -> JsonB {
    let mut task_map = HashMap::new();
    task_map.insert("task", task);
    let task_json = json!(task_map);
    JsonB(crate::bindings::transformers::transform(
        &task_json, &args.0, &inputs,
    ))
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;
    use crate::orm::algorithm::Algorithm;
    use crate::orm::dataset::{load_diabetes, load_digits};
    use crate::orm::runtime::Runtime;
    use crate::orm::sampling::Sampling;
    use crate::orm::Hyperparams;

    #[pg_test]
    fn test_project_lifecycle() {
        assert!(Project::create("test", Task::regression).id > 0);
        assert!(Project::find(1).unwrap().id > 0);
    }

    #[pg_test]
    fn test_snapshot_lifecycle() {
        load_diabetes(Some(25));
        let snapshot = Snapshot::create("pgml.diabetes", "target", 0.5, Sampling::last);
        assert!(snapshot.id > 0);
    }

    #[pg_test]
    fn test_train_regression() {
        load_diabetes(None);

        // Modify postgresql.conf and add shared_preload_libraries = 'pgml'
        // to test deployments.
        let setting =
            Spi::get_one::<String>("select setting from pg_settings where name = 'data_directory'");

        info!("Data directory: {}", setting.unwrap());

        for runtime in [Runtime::python, Runtime::rust] {
            let result: Vec<(String, String, String, bool)> = train(
                "Test project",
                Some(Task::regression),
                Some("pgml.diabetes"),
                Some("target"),
                Algorithm::linear,
                JsonB(serde_json::Value::Object(Hyperparams::new())),
                None,
                JsonB(serde_json::Value::Object(Hyperparams::new())),
                JsonB(serde_json::Value::Object(Hyperparams::new())),
                0.25,
                Sampling::last,
                Some(runtime),
                Some(true),
            )
            .collect();

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, String::from("Test project"));
            assert_eq!(result[0].1, String::from("regression"));
            assert_eq!(result[0].2, String::from("linear"));
            // assert_eq!(result[0].3, true);
        }
    }

    #[pg_test]
    fn test_train_classification() {
        load_digits(None);

        // Modify postgresql.conf and add shared_preload_libraries = 'pgml'
        // to test deployments.
        let setting =
            Spi::get_one::<String>("select setting from pg_settings where name = 'data_directory'");

        info!("Data directory: {}", setting.unwrap());

        for runtime in [Runtime::python, Runtime::rust] {
            let result: Vec<(String, String, String, bool)> = train(
                "Test project 2",
                Some(Task::classification),
                Some("pgml.digits"),
                Some("target"),
                Algorithm::xgboost,
                JsonB(serde_json::Value::Object(Hyperparams::new())),
                None,
                JsonB(serde_json::Value::Object(Hyperparams::new())),
                JsonB(serde_json::Value::Object(Hyperparams::new())),
                0.25,
                Sampling::last,
                Some(runtime),
                Some(true),
            )
            .collect();

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, String::from("Test project 2"));
            assert_eq!(result[0].1, String::from("classification"));
            assert_eq!(result[0].2, String::from("xgboost"));
            // assert_eq!(result[0].3, true);
        }
    }
}
