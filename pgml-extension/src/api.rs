use std::collections::HashMap;
use std::fmt::Write;
use std::str::FromStr;

use pgx::iter::{SetOfIterator, TableIterator};
use pgx::*;

#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde_json::json;

#[cfg(feature = "python")]
use crate::bindings::sklearn::package_version;
use crate::orm::*;

#[cfg(feature = "python")]
#[pg_extern]
pub fn validate_python_dependencies() -> bool {
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

    info!(
        "Scikit-learn {}, XGBoost {}, LightGBM {}, NumPy {}",
        package_version("sklearn"),
        package_version("xgboost"),
        package_version("lightgbm"),
        package_version("numpy"),
    );

    true
}

#[cfg(not(feature = "python"))]
#[pg_extern]
pub fn validate_python_dependencies() {}

#[cfg(feature = "python")]
#[pg_extern]
pub fn python_package_version(name: &str) -> String {
    package_version(name)
}

#[cfg(not(feature = "python"))]
#[pg_extern]
pub fn python_package_version(name: &str) {
    error!("Python is not installed, recompile with `--features python`");
}

#[pg_extern]
pub fn validate_shared_library() {
    let shared_preload_libraries: String = Spi::get_one(
        "SELECT setting
         FROM pg_settings
         WHERE name = 'shared_preload_libraries'
         LIMIT 1",
    )
    .unwrap();

    if !shared_preload_libraries.contains("pgml") {
        error!("`pgml` must be added to `shared_preload_libraries` setting or models cannot be deployed");
    }
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
    task: default!(Option<Task>, "NULL"),
    relation_name: default!(Option<&str>, "NULL"),
    y_column_name: default!(Option<&str>, "NULL"),
    algorithm: default!(Algorithm, "'linear'"),
    hyperparams: default!(JsonB, "'{}'"),
    search: default!(Option<Search>, "NULL"),
    search_params: default!(JsonB, "'{}'"),
    search_args: default!(JsonB, "'{}'"),
    test_size: default!(f32, 0.25),
    test_sampling: default!(Sampling, "'last'"),
    runtime: default!(Option<Runtime>, "NULL"),
    automatic_deploy: default!(Option<bool>, true),
    materialize_snapshot: default!(bool, false),
) -> TableIterator<
    'static,
    (
        name!(project, String),
        name!(task, String),
        name!(algorithm, String),
        name!(deployed, bool),
    ),
> {
    train_joint(
        project_name,
        task,
        relation_name,
        y_column_name.map(|y_column_name| vec![y_column_name.to_string()]),
        algorithm,
        hyperparams,
        search,
        search_params,
        search_args,
        test_size,
        test_sampling,
        runtime,
        automatic_deploy,
        materialize_snapshot,
    )
}

#[allow(clippy::too_many_arguments)]
#[pg_extern]
fn train_joint(
    project_name: &str,
    task: default!(Option<Task>, "NULL"),
    relation_name: default!(Option<&str>, "NULL"),
    y_column_name: default!(Option<Vec<String>>, "NULL"),
    algorithm: default!(Algorithm, "'linear'"),
    hyperparams: default!(JsonB, "'{}'"),
    search: default!(Option<Search>, "NULL"),
    search_params: default!(JsonB, "'{}'"),
    search_args: default!(JsonB, "'{}'"),
    test_size: default!(f32, 0.25),
    test_sampling: default!(Sampling, "'last'"),
    runtime: default!(Option<Runtime>, "NULL"),
    automatic_deploy: default!(Option<bool>, true),
    materialize_snapshot: default!(bool, false),
) -> TableIterator<
    'static,
    (
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

            info!("Using existing snapshot from {}", snapshot.snapshot_name(),);

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
                materialize_snapshot,
            );

            if materialize_snapshot {
                info!(
                    "Snapshot of table \"{}\" created and saved in {}",
                    relation_name,
                    snapshot.snapshot_name(),
                );
            }

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
        project.deploy(model.id);
    }

    TableIterator::new(
        vec![(
            project.name,
            project.task.to_string(),
            model.algorithm.to_string(),
            deploy,
        )]
        .into_iter(),
    )
}

#[pg_extern]
fn deploy(
    project_name: &str,
    strategy: Strategy,
    algorithm: default!(Option<Algorithm>, "NULL"),
) -> TableIterator<
    'static,
    (
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

    let project = Project::find(project_id).unwrap();
    project.deploy(model_id);

    TableIterator::new(
        vec![(project_name.to_string(), strategy.to_string(), algorithm)].into_iter(),
    )
}

#[pg_extern(strict, name = "predict")]
fn predict(project_name: &str, features: Vec<f32>) -> f32 {
    predict_model(Project::get_deployed_model_id(project_name), features)
}

#[pg_extern(strict, name = "predict_proba")]
fn predict_proba(project_name: &str, features: Vec<f32>) -> Vec<f32> {
    predict_model_proba(Project::get_deployed_model_id(project_name), features)
}

#[pg_extern(strict, name = "predict_joint")]
fn predict_joint(project_name: &str, features: Vec<f32>) -> Vec<f32> {
    predict_model_joint(Project::get_deployed_model_id(project_name), features)
}

#[pg_extern(strict, name = "predict_batch")]
fn predict_batch(project_name: &str, features: Vec<f32>) -> SetOfIterator<'static, f32> {
    SetOfIterator::new(
        predict_model_batch(Project::get_deployed_model_id(project_name), features).into_iter(),
    )
}

#[pg_extern(strict, name = "predict")]
fn predict_model(model_id: i64, features: Vec<f32>) -> f32 {
    Model::find_cached(model_id).predict(&features)
}

#[pg_extern(strict, name = "predict_proba")]
fn predict_model_proba(model_id: i64, features: Vec<f32>) -> Vec<f32> {
    Model::find_cached(model_id).predict_proba(&features)
}

#[pg_extern(strict, name = "predict_joint")]
fn predict_model_joint(model_id: i64, features: Vec<f32>) -> Vec<f32> {
    Model::find_cached(model_id).predict_joint(&features)
}

#[pg_extern(strict, name = "predict_batch")]
fn predict_model_batch(model_id: i64, features: Vec<f32>) -> Vec<f32> {
    Model::find_cached(model_id).predict_batch(&features)
}

#[pg_extern]
fn snapshot(
    relation_name: &str,
    y_column_name: &str,
    test_size: default!(f32, 0.25),
    test_sampling: default!(Sampling, "'last'"),
) -> TableIterator<'static, (name!(relation, String), name!(y_column_name, String))> {
    Snapshot::create(
        relation_name,
        vec![y_column_name.to_string()],
        test_size,
        test_sampling,
        true,
    );
    TableIterator::new(vec![(relation_name.to_string(), y_column_name.to_string())].into_iter())
}

#[pg_extern]
fn load_dataset(
    source: &str,
    limit: default!(Option<i64>, "NULL"),
) -> TableIterator<'static, (name!(table_name, String), name!(rows, i64))> {
    // cast limit since pgx doesn't support usize
    let limit: Option<usize> = limit.map(|limit| limit.try_into().unwrap());
    let (name, rows) = match source {
        "breast_cancer" => crate::orm::dataset::load_breast_cancer(limit),
        "diabetes" => crate::orm::dataset::load_diabetes(limit),
        "digits" => crate::orm::dataset::load_digits(limit),
        "iris" => crate::orm::dataset::load_iris(limit),
        "linnerud" => crate::orm::dataset::load_linnerud(limit),
        "wine" => crate::orm::dataset::load_wine(limit),
        _ => error!("Unknown source: `{source}`"),
    };

    TableIterator::new(vec![(name, rows)].into_iter())
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

#[cfg(feature = "python")]
#[pg_extern(name = "sklearn_f1_score")]
pub fn sklearn_f1_score(ground_truth: Vec<f32>, y_hat: Vec<f32>) -> f32 {
    crate::bindings::sklearn::f1(&ground_truth, &y_hat)
}

#[cfg(feature = "python")]
#[pg_extern(name = "sklearn_r2_score")]
pub fn sklearn_r2_score(ground_truth: Vec<f32>, y_hat: Vec<f32>) -> f32 {
    crate::bindings::sklearn::r2(&ground_truth, &y_hat)
}

#[cfg(feature = "python")]
#[pg_extern(name = "sklearn_regression_metrics")]
pub fn sklearn_regression_metrics(ground_truth: Vec<f32>, y_hat: Vec<f32>) -> JsonB {
    JsonB(
        serde_json::from_str(
            &serde_json::to_string(&crate::bindings::sklearn::regression_metrics(
                &ground_truth,
                &y_hat,
            ))
            .unwrap(),
        )
        .unwrap(),
    )
}

#[cfg(feature = "python")]
#[pg_extern(name = "sklearn_classification_metrics")]
pub fn sklearn_classification_metrics(
    ground_truth: Vec<f32>,
    y_hat: Vec<f32>,
    num_classes: i64,
) -> JsonB {
    JsonB(
        serde_json::from_str(
            &serde_json::to_string(&crate::bindings::sklearn::classification_metrics(
                &ground_truth,
                &y_hat,
                num_classes as usize,
            ))
            .unwrap(),
        )
        .unwrap(),
    )
}

#[pg_extern]
pub fn dump_all(path: &str) {
    let p = std::path::Path::new(path).join("projects.csv");
    Spi::run(&format!(
        "COPY pgml.projects TO '{}' CSV HEADER",
        p.to_str().unwrap()
    ));

    let p = std::path::Path::new(path).join("snapshots.csv");
    Spi::run(&format!(
        "COPY pgml.snapshots TO '{}' CSV HEADER",
        p.to_str().unwrap()
    ));

    let p = std::path::Path::new(path).join("models.csv");
    Spi::run(&format!(
        "COPY pgml.models TO '{}' CSV HEADER",
        p.to_str().unwrap()
    ));

    let p = std::path::Path::new(path).join("files.csv");
    Spi::run(&format!(
        "COPY pgml.files TO '{}' CSV HEADER",
        p.to_str().unwrap()
    ));

    let p = std::path::Path::new(path).join("deployments.csv");
    Spi::run(&format!(
        "COPY pgml.deployments TO '{}' CSV HEADER",
        p.to_str().unwrap()
    ));
}

#[pg_extern]
pub fn load_all(path: &str) {
    let p = std::path::Path::new(path).join("projects.csv");
    Spi::run(&format!(
        "COPY pgml.projects FROM '{}' CSV HEADER",
        p.to_str().unwrap()
    ));

    let p = std::path::Path::new(path).join("snapshots.csv");
    Spi::run(&format!(
        "COPY pgml.snapshots FROM '{}' CSV HEADER",
        p.to_str().unwrap()
    ));

    let p = std::path::Path::new(path).join("models.csv");
    Spi::run(&format!(
        "COPY pgml.models FROM '{}' CSV HEADER",
        p.to_str().unwrap()
    ));

    let p = std::path::Path::new(path).join("files.csv");
    Spi::run(&format!(
        "COPY pgml.files FROM '{}' CSV HEADER",
        p.to_str().unwrap()
    ));

    let p = std::path::Path::new(path).join("deployments.csv");
    Spi::run(&format!(
        "COPY pgml.deployments FROM '{}' CSV HEADER",
        p.to_str().unwrap()
    ));
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;
    use crate::orm::algorithm::Algorithm;
    use crate::orm::dataset::{load_breast_cancer, load_diabetes, load_digits};
    use crate::orm::runtime::Runtime;
    use crate::orm::sampling::Sampling;
    use crate::orm::Hyperparams;

    #[pg_test]
    fn test_project_lifecycle() {
        let project = Project::create("test", Task::regression);
        assert!(project.id > 0);
        assert!(Project::find(project.id).unwrap().id > 0);
    }

    #[pg_test]
    fn test_snapshot_lifecycle() {
        load_diabetes(Some(25));

        let snapshot = Snapshot::create(
            "pgml.diabetes",
            vec!["target".to_string()],
            0.5,
            Sampling::last,
            true,
        );
        assert!(snapshot.id > 0);
    }

    #[pg_test]
    fn test_not_fully_qualified_table() {
        load_diabetes(Some(25));

        let result = std::panic::catch_unwind(|| {
            let _snapshot = Snapshot::create(
                "diabetes",
                vec!["target".to_string()],
                0.5,
                Sampling::last,
                true,
            );
        });

        assert!(result.is_err());
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
                false,
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
    fn test_train_multiclass_classification() {
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
                false,
            )
            .collect();

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, String::from("Test project 2"));
            assert_eq!(result[0].1, String::from("classification"));
            assert_eq!(result[0].2, String::from("xgboost"));
            // assert_eq!(result[0].3, true);
        }
    }

    #[pg_test]
    fn test_train_binary_classification() {
        load_breast_cancer(None);

        // Modify postgresql.conf and add shared_preload_libraries = 'pgml'
        // to test deployments.
        let setting =
            Spi::get_one::<String>("select setting from pg_settings where name = 'data_directory'");

        info!("Data directory: {}", setting.unwrap());

        for runtime in [Runtime::python, Runtime::rust] {
            let result: Vec<(String, String, String, bool)> = train(
                "Test project 3",
                Some(Task::classification),
                Some("pgml.breast_cancer"),
                Some("malignant"),
                Algorithm::xgboost,
                JsonB(serde_json::Value::Object(Hyperparams::new())),
                None,
                JsonB(serde_json::Value::Object(Hyperparams::new())),
                JsonB(serde_json::Value::Object(Hyperparams::new())),
                0.25,
                Sampling::last,
                Some(runtime),
                Some(true),
                true,
            )
            .collect();

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, String::from("Test project 3"));
            assert_eq!(result[0].1, String::from("classification"));
            assert_eq!(result[0].2, String::from("xgboost"));
            // assert_eq!(result[0].3, true);
        }
    }

    #[pg_test]
    fn test_dump_load() {
        dump_all("/tmp");
        load_all("/tmp");
    }
}
