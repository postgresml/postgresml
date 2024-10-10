use std::fmt::Write;
use std::str::FromStr;

use ndarray::Zip;
use pgrx::iter::{SetOfIterator, TableIterator};
use pgrx::*;

#[cfg(feature = "python")]
use serde_json::json;

#[cfg(feature = "python")]
use crate::orm::*;

macro_rules! unwrap_or_error {
    ($i:expr) => {
        match $i {
            Ok(v) => v,
            Err(e) => error!("{e}"),
        }
    };
}

#[cfg(feature = "python")]
#[pg_extern]
pub fn activate_venv(venv: &str) -> bool {
    unwrap_or_error!(crate::bindings::python::activate_venv(venv))
}

#[cfg(feature = "python")]
#[pg_extern(immutable, parallel_safe)]
pub fn validate_python_dependencies() -> bool {
    unwrap_or_error!(crate::bindings::python::validate_dependencies())
}

#[cfg(not(feature = "python"))]
#[pg_extern]
pub fn validate_python_dependencies() {}

#[cfg(feature = "python")]
#[pg_extern]
pub fn python_package_version(name: &str) -> String {
    unwrap_or_error!(crate::bindings::python::package_version(name))
}

#[cfg(not(feature = "python"))]
#[pg_extern]
pub fn python_package_version(name: &str) {
    error!("Python is not installed, recompile with `--features python`");
}

#[cfg(feature = "python")]
#[pg_extern]
pub fn python_pip_freeze() -> TableIterator<'static, (name!(package, String),)> {
    unwrap_or_error!(crate::bindings::python::pip_freeze())
}

#[cfg(feature = "python")]
#[pg_extern]
fn python_version() -> String {
    unwrap_or_error!(crate::bindings::python::version())
}

#[cfg(not(feature = "python"))]
#[pg_extern]
pub fn python_version() -> String {
    String::from("Python is not installed, recompile with `--features python`")
}

#[pg_extern]
pub fn validate_shared_library() {
    let shared_preload_libraries: String = Spi::get_one(
        "SELECT setting
         FROM pg_settings
         WHERE name = 'shared_preload_libraries'
         LIMIT 1",
    )
    .unwrap()
    .unwrap();

    if !shared_preload_libraries.contains("pgml") {
        error!("`pgml` must be added to `shared_preload_libraries` setting or models cannot be deployed");
    }
}

#[pg_extern(immutable, parallel_safe)]
fn version() -> String {
    format!("{} ({})", crate::VERSION, crate::COMMIT)
}

#[allow(clippy::too_many_arguments)]
#[pg_extern]
fn train(
    project_name: &str,
    task: default!(Option<&str>, "NULL"),
    relation_name: default!(Option<&str>, "NULL"),
    y_column_name: default!(Option<&str>, "NULL"),
    algorithm: default!(Algorithm, "'linear'"),
    hyperparams: default!(JsonB, "'{}'"),
    search: default!(Option<Search>, "NULL"),
    search_params: default!(JsonB, "'{}'"),
    search_args: default!(JsonB, "'{}'"),
    test_size: default!(f32, 0.25),
    test_sampling: default!(Sampling, "'stratified'"),
    runtime: default!(Option<Runtime>, "NULL"),
    automatic_deploy: default!(Option<bool>, true),
    materialize_snapshot: default!(bool, false),
    preprocess: default!(JsonB, "'{}'"),
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
        preprocess,
    )
}

#[allow(clippy::too_many_arguments)]
#[pg_extern]
fn train_joint(
    project_name: &str,
    task: default!(Option<&str>, "NULL"),
    relation_name: default!(Option<&str>, "NULL"),
    y_column_name: default!(Option<Vec<String>>, "NULL"),
    algorithm: default!(Algorithm, "'linear'"),
    hyperparams: default!(JsonB, "'{}'"),
    search: default!(Option<Search>, "NULL"),
    search_params: default!(JsonB, "'{}'"),
    search_args: default!(JsonB, "'{}'"),
    test_size: default!(f32, 0.25),
    test_sampling: default!(Sampling, "'stratified'"),
    runtime: default!(Option<Runtime>, "NULL"),
    automatic_deploy: default!(Option<bool>, true),
    materialize_snapshot: default!(bool, false),
    preprocess: default!(JsonB, "'{}'"),
) -> TableIterator<
    'static,
    (
        name!(project, String),
        name!(task, String),
        name!(algorithm, String),
        name!(deployed, bool),
    ),
> {
    let task = task.map(|t| Task::from_str(t).unwrap());
    let project = match Project::find_by_name(project_name) {
        Some(project) => project,
        None => Project::create(
            project_name,
            match task {
                Some(task) => task,
                None => error!(
                    "Project `{}` does not exist. To create a new project, you must specify a `task`.",
                    project_name
                ),
            },
        ),
    };

    if task.is_some() && task.unwrap() != project.task {
        error!(
            "Project `{:?}` already exists with a different task: `{:?}`. Create a new project instead.",
            project.name, project.task
        );
    }

    let mut snapshot = match relation_name {
        None => {
            let snapshot = project.last_snapshot().expect(
                "You must pass a `relation_name` and `y_column_name` to snapshot the first time you train a model.",
            );

            notice!("Using existing snapshot from {}", snapshot.snapshot_name(),);

            snapshot
        }

        Some(relation_name) => {
            notice!(
                "Snapshotting table \"{}\", this may take a little while...",
                relation_name
            );

            if project.task.is_supervised() && y_column_name.is_none() {
                error!("You must pass a `y_column_name` when you pass a `relation_name` for a supervised task.");
            }

            let snapshot = Snapshot::create(
                relation_name,
                y_column_name,
                test_size,
                test_sampling,
                materialize_snapshot,
                preprocess,
            );

            if materialize_snapshot {
                notice!(
                    "Snapshot of table \"{}\" created and saved in {}",
                    relation_name,
                    snapshot.snapshot_name(),
                );
            }

            snapshot
        }
    };

    // fix up default algorithm for clustering
    let algorithm = if algorithm == Algorithm::linear && project.task == Task::clustering {
        Algorithm::kmeans
    } else if algorithm == Algorithm::linear && project.task == Task::decomposition {
        Algorithm::pca
    } else {
        algorithm
    };

    // # Default repeatable random state when possible
    // let algorithm = Model.algorithm_from_name_and_task(algorithm, task);
    // if "random_state" in algorithm().get_params() and "random_state" not in hyperparams:
    //     hyperparams["random_state"] = 0
    let model = Model::create(
        &project,
        &mut snapshot,
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
        // Deploy only if metrics are better than previous model, or if its the first model
        Some(true) | None => {
            if let Ok(Some(deployed_metrics)) = deployed_metrics {
                if let Some(deployed_metrics_obj) = deployed_metrics.0.as_object() {
                    let default_target_metric = project.task.default_target_metric();
                    let deployed_metric = deployed_metrics_obj
                        .get(&default_target_metric)
                        .and_then(|v| v.as_f64());
                    notice!(
                        "Comparing to deployed model {}: {:?}",
                        default_target_metric,
                        deployed_metric
                    );
                    let new_metric = new_metrics.get(&default_target_metric).and_then(|v| v.as_f64());

                    match (deployed_metric, new_metric) {
                        (Some(deployed), Some(new)) => {
                            // only compare metrics when both new and old model have metrics to compare
                            if project.task.value_is_better(deployed, new) {
                                warning!(
                                    "New model's {} is not better than current model. New: {}, Current {}",
                                    &default_target_metric,
                                    new,
                                    deployed
                                );
                                deploy = false;
                            }
                        }
                        (None, None) => {
                            warning!("No metrics available for both deployed and new model. Deploying new model.")
                        }
                        (Some(_deployed), None) => {
                            warning!("No metrics for new model. Retaining old model.");
                            deploy = false;
                        }
                        (None, Some(_new)) => warning!("No metrics for deployed model. Deploying new model."),
                    }
                } else {
                    warning!("Failed to parse deployed model metrics. Check data types of model metadata on pgml.models.metrics");
                    deploy = false;
                }
            }
        }
        Some(false) => {
            warning!("Automatic deployment disabled via configuration.");
            deploy = false;
        }
    };
    if deploy {
        project.deploy(model.id, Strategy::new_score);
    } else {
        warning!("Not deploying newly trained model.");
    }

    TableIterator::new(vec![(
        project.name,
        project.task.to_string(),
        model.algorithm.to_string(),
        deploy,
    )])
}

#[pg_extern(name = "deploy")]
fn deploy_model(
    model_id: i64,
) -> TableIterator<
    'static,
    (
        name!(project, String),
        name!(strategy, String),
        name!(algorithm, String),
    ),
> {
    let model = unwrap_or_error!(Model::find_cached(model_id));

    let project_id = Spi::get_one_with_args::<i64>(
        "SELECT projects.id from pgml.projects JOIN pgml.models ON models.project_id = projects.id WHERE models.id = $1",
        vec![(PgBuiltInOids::INT8OID.oid(), model_id.into_datum())],
    )
        .unwrap();

    let project_id = project_id.unwrap_or_else(|| error!("Project does not exist."));

    let project = Project::find(project_id).unwrap();
    project.deploy(model_id, Strategy::specific);

    TableIterator::new(vec![(
        project.name,
        Strategy::specific.to_string(),
        model.algorithm.to_string(),
    )])
}

#[pg_extern(name = "deploy")]
fn deploy_strategy(
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
    )
    .unwrap();

    let project_id = project_id.unwrap_or_else(|| error!("Project named `{}` does not exist.", project_name));

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
        Strategy::best_score => {
            let _ = write!(sql, "{predicate}\n{}", task.default_target_metric_sql_order());
        }

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
        _ => error!("invalid strategy"),
    }
    sql += "\nLIMIT 1";
    let (model_id, algorithm) =
        Spi::get_two_with_args::<i64, String>(&sql, vec![(PgBuiltInOids::TEXTOID.oid(), project_name.into_datum())])
            .unwrap();
    let model_id = model_id.expect("No qualified models exist for this deployment.");
    let algorithm = algorithm.expect("No qualified models exist for this deployment.");

    let project = Project::find(project_id).unwrap();
    project.deploy(model_id, strategy);

    TableIterator::new(vec![(project_name.to_string(), strategy.to_string(), algorithm)])
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict")]
fn predict_f32(project_name: &str, features: Vec<f32>) -> f32 {
    predict_model(Project::get_deployed_model_id(project_name), features)
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict")]
fn predict_f64(project_name: &str, features: Vec<f64>) -> f32 {
    predict_f32(project_name, features.iter().map(|&i| i as f32).collect())
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict")]
fn predict_i16(project_name: &str, features: Vec<i16>) -> f32 {
    predict_f32(project_name, features.iter().map(|&i| i as f32).collect())
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict")]
fn predict_i32(project_name: &str, features: Vec<i32>) -> f32 {
    predict_f32(project_name, features.iter().map(|&i| i as f32).collect())
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict")]
fn predict_i64(project_name: &str, features: Vec<i64>) -> f32 {
    predict_f32(project_name, features.iter().map(|&i| i as f32).collect())
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict")]
fn predict_bool(project_name: &str, features: Vec<bool>) -> f32 {
    predict_f32(project_name, features.iter().map(|&i| i as u8 as f32).collect())
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict_proba")]
fn predict_proba(project_name: &str, features: Vec<f32>) -> Vec<f32> {
    predict_model_proba(Project::get_deployed_model_id(project_name), features)
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict_joint")]
fn predict_joint(project_name: &str, features: Vec<f32>) -> Vec<f32> {
    predict_model_joint(Project::get_deployed_model_id(project_name), features)
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict_batch")]
fn predict_batch(project_name: &str, features: Vec<f32>) -> SetOfIterator<'static, f32> {
    SetOfIterator::new(predict_model_batch(
        Project::get_deployed_model_id(project_name),
        features,
    ))
}

#[pg_extern(immutable, parallel_safe, strict, name = "decompose")]
fn decompose(project_name: &str, vector: Vec<f32>) -> Vec<f32> {
    let model_id = Project::get_deployed_model_id(project_name);
    let model = unwrap_or_error!(Model::find_cached(model_id));
    unwrap_or_error!(model.decompose(&vector))
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict")]
fn predict_row(project_name: &str, row: pgrx::datum::AnyElement) -> f32 {
    predict_model_row(Project::get_deployed_model_id(project_name), row)
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict")]
fn predict_model(model_id: i64, features: Vec<f32>) -> f32 {
    let model = unwrap_or_error!(Model::find_cached(model_id));
    unwrap_or_error!(model.predict(&features))
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict_proba")]
fn predict_model_proba(model_id: i64, features: Vec<f32>) -> Vec<f32> {
    let model = unwrap_or_error!(Model::find_cached(model_id));
    unwrap_or_error!(model.predict_proba(&features))
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict_joint")]
fn predict_model_joint(model_id: i64, features: Vec<f32>) -> Vec<f32> {
    let model = unwrap_or_error!(Model::find_cached(model_id));
    unwrap_or_error!(model.predict_joint(&features))
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict_batch")]
fn predict_model_batch(model_id: i64, features: Vec<f32>) -> Vec<f32> {
    let model = unwrap_or_error!(Model::find_cached(model_id));
    unwrap_or_error!(model.predict_batch(&features))
}

#[pg_extern(immutable, parallel_safe, strict, name = "predict")]
fn predict_model_row(model_id: i64, row: pgrx::datum::AnyElement) -> f32 {
    let model = unwrap_or_error!(Model::find_cached(model_id));
    let snapshot = &model.snapshot;
    let numeric_encoded_features = model.numeric_encode_features(&[row]);
    let features_width = snapshot.features_width();
    let mut processed = vec![0_f32; features_width];

    let feature_data = ndarray::ArrayView2::from_shape((1, features_width), &numeric_encoded_features).unwrap();

    Zip::from(feature_data.columns())
        .and(&snapshot.feature_positions)
        .for_each(|data, position| {
            let column = &snapshot.columns[position.column_position - 1];
            column.preprocess(&data, &mut processed, features_width, position.row_position);
        });
    unwrap_or_error!(model.predict(&processed))
}

#[pg_extern]
fn snapshot(
    relation_name: &str,
    y_column_name: &str,
    test_size: default!(f32, 0.25),
    test_sampling: default!(Sampling, "'stratified'"),
    preprocess: default!(JsonB, "'{}'"),
) -> TableIterator<'static, (name!(relation, String), name!(y_column_name, String))> {
    Snapshot::create(
        relation_name,
        Some(vec![y_column_name.to_string()]),
        test_size,
        test_sampling,
        true,
        preprocess,
    );
    TableIterator::new(vec![(relation_name.to_string(), y_column_name.to_string())])
}

#[pg_extern]
fn load_dataset(
    source: &str,
    subset: default!(Option<String>, "NULL"),
    limit: default!(Option<i64>, "NULL"),
    kwargs: default!(JsonB, "'{}'"),
) -> TableIterator<'static, (name!(table_name, String), name!(rows, i64))> {
    // cast limit since pgrx doesn't support usize
    let limit: Option<usize> = limit.map(|limit| limit.try_into().unwrap());
    let (name, rows) = match source {
        "breast_cancer" => dataset::load_breast_cancer(limit),
        "diabetes" => dataset::load_diabetes(limit),
        "digits" => dataset::load_digits(limit),
        "iris" => dataset::load_iris(limit),
        "linnerud" => dataset::load_linnerud(limit),
        "wine" => dataset::load_wine(limit),
        _ => {
            let rows = match crate::bindings::transformers::load_dataset(source, subset, limit, &kwargs.0) {
                Ok(rows) => rows,
                Err(e) => error!("{e}"),
            };
            (source.into(), rows as i64)
        }
    };

    TableIterator::new(vec![(name, rows)])
}

#[cfg(all(feature = "python", not(feature = "use_as_lib")))]
#[pg_extern(immutable, parallel_safe, name = "embed")]
pub fn embed(transformer: &str, text: &str, kwargs: default!(JsonB, "'{}'")) -> Vec<f32> {
    match crate::bindings::transformers::embed(transformer, vec![text], &kwargs.0) {
        Ok(output) => output.first().unwrap().to_vec(),
        Err(e) => error!("{e}"),
    }
}

#[cfg(all(feature = "python", not(feature = "use_as_lib")))]
#[pg_extern(immutable, parallel_safe, name = "embed")]
pub fn embed_batch(
    transformer: &str,
    inputs: Vec<&str>,
    kwargs: default!(JsonB, "'{}'"),
) -> SetOfIterator<'static, Vec<f32>> {
    match crate::bindings::transformers::embed(transformer, inputs, &kwargs.0) {
        Ok(output) => SetOfIterator::new(output),
        Err(e) => error!("{e}"),
    }
}

#[cfg(all(feature = "python", not(feature = "use_as_lib")))]
#[pg_extern(immutable, parallel_safe, name = "rank")]
pub fn rank(
    transformer: &str,
    query: &str,
    documents: Vec<&str>,
    kwargs: default!(JsonB, "'{}'"),
) -> TableIterator<'static, (name!(corpus_id, i64), name!(score, f64), name!(text, Option<String>))> {
    match crate::bindings::transformers::rank(transformer, query, documents, &kwargs.0) {
        Ok(output) => TableIterator::new(output.into_iter().map(|x| (x.corpus_id, x.score, x.text))),
        Err(e) => error!("{e}"),
    }
}

/// Clears the GPU cache.
///
/// # Arguments
///
/// * `memory_usage` - Optional parameter indicating the memory usage percentage (0.0 -> 1.0)
///
/// # Returns
///
/// Returns `true` if the GPU cache was successfully cleared, `false` otherwise.
/// # Example
///
/// ```postgresql
/// SELECT pgml.clear_gpu_cache(memory_usage => 0.5);
/// ```
#[cfg(all(feature = "python", not(feature = "use_as_lib")))]
#[pg_extern(immutable, parallel_safe, name = "clear_gpu_cache")]
pub fn clear_gpu_cache(memory_usage: default!(Option<f32>, "NULL")) -> bool {
    match crate::bindings::transformers::clear_gpu_cache(memory_usage) {
        Ok(success) => success,
        Err(e) => error!("{e}"),
    }
}

#[pg_extern(immutable, parallel_safe)]
pub fn chunk(
    splitter: &str,
    text: &str,
    kwargs: default!(JsonB, "'{}'"),
) -> TableIterator<'static, (name!(chunk_index, i64), name!(chunk, String))> {
    let chunks = match crate::bindings::langchain::chunk(splitter, text, &kwargs.0) {
        Ok(chunks) => chunks,
        Err(e) => error!("{e}"),
    };

    let chunks = chunks
        .into_iter()
        .enumerate()
        .map(|(i, chunk)| (i as i64 + 1, chunk))
        .collect::<Vec<(i64, String)>>();

    TableIterator::new(chunks)
}

#[cfg(all(feature = "python", not(feature = "use_as_lib")))]
#[pg_extern(immutable, parallel_safe, name = "transform")]
#[allow(unused_variables)] // cache is maintained for api compatibility
pub fn transform_json(
    task: JsonB,
    args: default!(JsonB, "'{}'"),
    inputs: default!(Vec<&str>, "ARRAY[]::TEXT[]"),
    cache: default!(bool, false),
) -> JsonB {
    if let Err(err) = crate::bindings::transformers::whitelist::verify_task(&task.0) {
        error!("{err}");
    }

    match crate::bindings::transformers::transform(&task.0, &args.0, inputs) {
        Ok(output) => JsonB(output),
        Err(e) => error!("{e}"),
    }
}

#[cfg(all(feature = "python", not(feature = "use_as_lib")))]
#[pg_extern(immutable, parallel_safe, name = "transform")]
#[allow(unused_variables)] // cache is maintained for api compatibility
pub fn transform_string(
    task: String,
    args: default!(JsonB, "'{}'"),
    inputs: default!(Vec<&str>, "ARRAY[]::TEXT[]"),
    cache: default!(bool, false),
) -> JsonB {
    let task_json = json!({ "task": task });
    if let Err(err) = crate::bindings::transformers::whitelist::verify_task(&task_json) {
        error!("{err}");
    }
    match crate::bindings::transformers::transform(&task_json, &args.0, inputs) {
        Ok(output) => JsonB(output),
        Err(e) => error!("{e}"),
    }
}

#[cfg(all(feature = "python", not(feature = "use_as_lib")))]
#[pg_extern(immutable, parallel_safe, name = "transform")]
#[allow(unused_variables)] // cache is maintained for api compatibility
pub fn transform_conversational_json(
    task: JsonB,
    args: default!(JsonB, "'{}'"),
    inputs: default!(Vec<JsonB>, "ARRAY[]::JSONB[]"),
    cache: default!(bool, false),
) -> JsonB {
    if !task.0["task"].as_str().is_some_and(|v| v == "conversational") {
        error!("ARRAY[]::JSONB inputs for transform should only be used with a conversational task");
    }
    if let Err(err) = crate::bindings::transformers::whitelist::verify_task(&task.0) {
        error!("{err}");
    }
    match crate::bindings::transformers::transform(&task.0, &args.0, inputs) {
        Ok(output) => JsonB(output),
        Err(e) => error!("{e}"),
    }
}

#[cfg(all(feature = "python", not(feature = "use_as_lib")))]
#[pg_extern(immutable, parallel_safe, name = "transform")]
#[allow(unused_variables)] // cache is maintained for api compatibility
pub fn transform_conversational_string(
    task: String,
    args: default!(JsonB, "'{}'"),
    inputs: default!(Vec<JsonB>, "ARRAY[]::JSONB[]"),
    cache: default!(bool, false),
) -> JsonB {
    if task != "conversational" {
        error!("ARRAY[]::JSONB inputs for transform should only be used with a conversational task");
    }
    let task_json = json!({ "task": task });
    if let Err(err) = crate::bindings::transformers::whitelist::verify_task(&task_json) {
        error!("{err}");
    }
    match crate::bindings::transformers::transform(&task_json, &args.0, inputs) {
        Ok(output) => JsonB(output),
        Err(e) => error!("{e}"),
    }
}

#[cfg(all(feature = "python", not(feature = "use_as_lib")))]
#[pg_extern(immutable, parallel_safe, name = "transform_stream")]
#[allow(unused_variables)] // cache is maintained for api compatibility
pub fn transform_stream_json(
    task: JsonB,
    args: default!(JsonB, "'{}'"),
    input: default!(&str, "''"),
    cache: default!(bool, false),
) -> SetOfIterator<'static, JsonB> {
    // We can unwrap this becuase if there is an error the current transaction is aborted in the map_err call
    let python_iter = crate::bindings::transformers::transform_stream_iterator(&task.0, &args.0, input)
        .map_err(|e| error!("{e}"))
        .unwrap();
    SetOfIterator::new(python_iter)
}

#[cfg(all(feature = "python", not(feature = "use_as_lib")))]
#[pg_extern(immutable, parallel_safe, name = "transform_stream")]
#[allow(unused_variables)] // cache is maintained for api compatibility
pub fn transform_stream_string(
    task: String,
    args: default!(JsonB, "'{}'"),
    input: default!(&str, "''"),
    cache: default!(bool, false),
) -> SetOfIterator<'static, JsonB> {
    let task_json = json!({ "task": task });
    // We can unwrap this becuase if there is an error the current transaction is aborted in the map_err call
    let python_iter = crate::bindings::transformers::transform_stream_iterator(&task_json, &args.0, input)
        .map_err(|e| error!("{e}"))
        .unwrap();
    SetOfIterator::new(python_iter)
}

#[cfg(all(feature = "python", not(feature = "use_as_lib")))]
#[pg_extern(immutable, parallel_safe, name = "transform_stream")]
#[allow(unused_variables)] // cache is maintained for api compatibility
pub fn transform_stream_conversational_json(
    task: JsonB,
    args: default!(JsonB, "'{}'"),
    inputs: default!(Vec<JsonB>, "ARRAY[]::JSONB[]"),
    cache: default!(bool, false),
) -> SetOfIterator<'static, JsonB> {
    if !task.0["task"].as_str().is_some_and(|v| v == "conversational") {
        error!("ARRAY[]::JSONB inputs for transform_stream should only be used with a conversational task");
    }
    // We can unwrap this becuase if there is an error the current transaction is aborted in the map_err call
    let python_iter = crate::bindings::transformers::transform_stream_iterator(&task.0, &args.0, inputs)
        .map_err(|e| error!("{e}"))
        .unwrap();
    SetOfIterator::new(python_iter)
}

#[cfg(all(feature = "python", not(feature = "use_as_lib")))]
#[pg_extern(immutable, parallel_safe, name = "transform_stream")]
#[allow(unused_variables)] // cache is maintained for api compatibility
pub fn transform_stream_conversational_string(
    task: String,
    args: default!(JsonB, "'{}'"),
    inputs: default!(Vec<JsonB>, "ARRAY[]::JSONB[]"),
    cache: default!(bool, false),
) -> SetOfIterator<'static, JsonB> {
    if task != "conversational" {
        error!("ARRAY::JSONB inputs for transform_stream should only be used with a conversational task");
    }
    let task_json = json!({ "task": task });
    // We can unwrap this becuase if there is an error the current transaction is aborted in the map_err call
    let python_iter = crate::bindings::transformers::transform_stream_iterator(&task_json, &args.0, inputs)
        .map_err(|e| error!("{e}"))
        .unwrap();
    SetOfIterator::new(python_iter)
}

#[cfg(feature = "python")]
#[pg_extern(immutable, parallel_safe, name = "generate")]
fn generate(project_name: &str, inputs: &str, config: default!(JsonB, "'{}'")) -> String {
    generate_batch(project_name, Vec::from([inputs]), config)
        .first()
        .unwrap()
        .to_string()
}

#[cfg(feature = "python")]
#[pg_extern(immutable, parallel_safe, name = "generate")]
fn generate_batch(project_name: &str, inputs: Vec<&str>, config: default!(JsonB, "'{}'")) -> Vec<String> {
    match crate::bindings::transformers::generate(Project::get_deployed_model_id(project_name), inputs, config) {
        Ok(output) => output,
        Err(e) => error!("{e}"),
    }
}

#[cfg(feature = "python")]
#[allow(clippy::too_many_arguments)]
#[pg_extern(parallel_safe)]
fn tune(
    project_name: &str,
    task: default!(Option<&str>, "NULL"),
    relation_name: default!(Option<&str>, "NULL"),
    _y_column_name: default!(Option<&str>, "NULL"),
    model_name: default!(Option<&str>, "NULL"),
    hyperparams: default!(JsonB, "'{}'"),
    test_size: default!(f32, 0.25),
    test_sampling: default!(Sampling, "'stratified'"),
    automatic_deploy: default!(Option<bool>, true),
    materialize_snapshot: default!(bool, false),
) -> TableIterator<
    'static,
    (
        name!(status, String),
        name!(task, String),
        name!(algorithm, String),
        name!(deployed, bool),
    ),
> {
    let task = task.map(|t| Task::from_str(t).unwrap());
    let preprocess = JsonB(serde_json::from_str("{}").unwrap());
    let project = match Project::find_by_name(project_name) {
        Some(project) => project,
        None => Project::create(
            project_name,
            match task {
                Some(task) => task,
                None => error!(
                    "Project `{}` does not exist. To create a new project, provide the task.",
                    project_name
                ),
            },
        ),
    };

    if task.is_some() && task.unwrap() != project.task {
        error!(
            "Project `{:?}` already exists with a different task: `{:?}`. Create a new project instead.",
            project.name, project.task
        );
    }

    let mut snapshot = match relation_name {
        None => {
            let snapshot = project.last_snapshot().expect(
                "You must pass a `relation_name` and `y_column_name` to snapshot the first time you train a model.",
            );

            notice!("Using existing snapshot from {}", snapshot.snapshot_name(),);

            snapshot
        }

        Some(relation_name) => {
            notice!(
                "Snapshotting table \"{}\", this may take a little while...",
                relation_name
            );

            let snapshot = Snapshot::create(
                relation_name,
                None,
                test_size,
                test_sampling,
                materialize_snapshot,
                preprocess,
            );

            if materialize_snapshot {
                notice!(
                    "Snapshot of table \"{}\" created and saved in {}",
                    relation_name,
                    snapshot.snapshot_name(),
                );
            }

            snapshot
        }
    };

    // algorithm will be transformers, stash the model_name in a hyperparam for v1 compatibility.
    let mut hyperparams = hyperparams.0.as_object().unwrap().clone();
    hyperparams.insert(String::from("model_name"), json!(model_name));
    hyperparams.insert(String::from("project_name"), json!(project_name));
    let hyperparams = JsonB(json!(hyperparams));

    // # Default repeatable random state when possible
    // let algorithm = Model.algorithm_from_name_and_task(algorithm, task);
    // if "random_state" in algorithm().get_params() and "random_state" not in hyperparams:
    //     hyperparams["random_state"] = 0
    let model = Model::finetune(&project, &mut snapshot, &hyperparams);
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
            if let Ok(Some(deployed_metrics)) = deployed_metrics {
                let deployed_metrics = deployed_metrics.0.as_object().unwrap();

                let deployed_value = deployed_metrics
                    .get(&project.task.default_target_metric())
                    .and_then(|value| value.as_f64())
                    .unwrap_or_default(); // Default to 0.0 if the key is not present or conversion fails

                // Get the value for the default target metric from new_metrics or provide a default value
                let new_value = new_metrics
                    .get(&project.task.default_target_metric())
                    .and_then(|value| value.as_f64())
                    .unwrap_or_default(); // Default to 0.0 if the key is not present or conversion fails

                if project.task.value_is_better(deployed_value, new_value) {
                    deploy = false;
                }
            }
        }

        Some(false) => deploy = false,
    };

    if deploy {
        project.deploy(model.id, Strategy::new_score);
    }

    TableIterator::new(vec![(
        project.name,
        project.task.to_string(),
        model.algorithm.to_string(),
        deploy,
    )])
}

#[cfg(feature = "python")]
#[pg_extern(name = "sklearn_f1_score")]
pub fn sklearn_f1_score(ground_truth: Vec<f32>, y_hat: Vec<f32>) -> f32 {
    unwrap_or_error!(crate::bindings::sklearn::f1(&ground_truth, &y_hat))
}

#[cfg(feature = "python")]
#[pg_extern(name = "sklearn_r2_score")]
pub fn sklearn_r2_score(ground_truth: Vec<f32>, y_hat: Vec<f32>) -> f32 {
    unwrap_or_error!(crate::bindings::sklearn::r2(&ground_truth, &y_hat))
}

#[cfg(feature = "python")]
#[pg_extern(name = "sklearn_regression_metrics")]
pub fn sklearn_regression_metrics(ground_truth: Vec<f32>, y_hat: Vec<f32>) -> JsonB {
    let metrics = unwrap_or_error!(crate::bindings::sklearn::regression_metrics(&ground_truth, &y_hat,));
    JsonB(json!(metrics))
}

#[cfg(feature = "python")]
#[pg_extern(name = "sklearn_classification_metrics")]
pub fn sklearn_classification_metrics(ground_truth: Vec<f32>, y_hat: Vec<f32>, num_classes: i64) -> JsonB {
    let metrics = unwrap_or_error!(crate::bindings::sklearn::classification_metrics(
        &ground_truth,
        &y_hat,
        num_classes as _
    ));

    JsonB(json!(metrics))
}

#[pg_extern]
pub fn dump_all(path: &str) {
    let p = std::path::Path::new(path).join("projects.csv");
    Spi::run(&format!("COPY pgml.projects TO '{}' CSV HEADER", p.to_str().unwrap())).unwrap();

    let p = std::path::Path::new(path).join("snapshots.csv");
    Spi::run(&format!("COPY pgml.snapshots TO '{}' CSV HEADER", p.to_str().unwrap())).unwrap();

    let p = std::path::Path::new(path).join("models.csv");
    Spi::run(&format!("COPY pgml.models TO '{}' CSV HEADER", p.to_str().unwrap())).unwrap();

    let p = std::path::Path::new(path).join("files.csv");
    Spi::run(&format!("COPY pgml.files TO '{}' CSV HEADER", p.to_str().unwrap())).unwrap();

    let p = std::path::Path::new(path).join("deployments.csv");
    Spi::run(&format!(
        "COPY pgml.deployments TO '{}' CSV HEADER",
        p.to_str().unwrap()
    ))
    .unwrap();
}

#[pg_extern]
pub fn load_all(path: &str) {
    let p = std::path::Path::new(path).join("projects.csv");
    Spi::run(&format!("COPY pgml.projects FROM '{}' CSV HEADER", p.to_str().unwrap())).unwrap();

    let p = std::path::Path::new(path).join("snapshots.csv");
    Spi::run(&format!(
        "COPY pgml.snapshots FROM '{}' CSV HEADER",
        p.to_str().unwrap()
    ))
    .unwrap();

    let p = std::path::Path::new(path).join("models.csv");
    Spi::run(&format!("COPY pgml.models FROM '{}' CSV HEADER", p.to_str().unwrap())).unwrap();

    let p = std::path::Path::new(path).join("files.csv");
    Spi::run(&format!("COPY pgml.files FROM '{}' CSV HEADER", p.to_str().unwrap())).unwrap();

    let p = std::path::Path::new(path).join("deployments.csv");
    Spi::run(&format!(
        "COPY pgml.deployments FROM '{}' CSV HEADER",
        p.to_str().unwrap()
    ))
    .unwrap();
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
    #[ignore = "requires model download"]
    fn readme_intro_translation() {
        let sql = "SELECT pgml.transform(
            'translation_en_to_fr',
            inputs => ARRAY[
                'Welcome to the future!',
                'Where have you been all this time?'
            ]
        ) AS french;";
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            {"translation_text": "Bienvenue à l'avenir!"},
            {"translation_text": "Où êtes-vous allé tout ce temps?"}
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_intro_sentiment_analysis() {
        let sql = "SELECT pgml.transform(
            task   => 'text-classification',
            inputs => ARRAY[
                'I love how amazingly simple ML has become!', 
                'I hate doing mundane and thankless tasks. ☹️'
            ]
        ) AS positivity;";
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            {"label": "POSITIVE", "score": 0.9995759129524232},
            {"label": "NEGATIVE", "score": 0.9903519749641418}
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_sentiment_analysis_specific_model() {
        let sql = r#"SELECT pgml.transform(
            inputs => ARRAY[
                'I love how amazingly simple ML has become!', 
                'I hate doing mundane and thankless tasks. ☹️'
            ],
            task  => '{"task": "text-classification", 
                    "model": "finiteautomata/bertweet-base-sentiment-analysis"
                    }'::JSONB
        ) AS positivity;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            {"label": "POS", "score": 0.992932200431826},
            {"label": "NEG", "score": 0.975599765777588}
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_sentiment_analysis_industry_specific_model() {
        let sql = r#"SELECT pgml.transform(
            inputs => ARRAY[
                'Stocks rallied and the British pound gained.', 
                'Stocks making the biggest moves midday: Nvidia, Palantir and more'
            ],
            task => '{"task": "text-classification", 
                    "model": "ProsusAI/finbert"
                    }'::JSONB
        ) AS market_sentiment;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            {"label": "positive", "score": 0.8983612656593323},
            {"label": "neutral", "score": 0.8062630891799927}
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_nli() {
        let sql = r#"SELECT pgml.transform(
            inputs => ARRAY[
                'A soccer game with multiple males playing. Some men are playing a sport.'
            ],
            task => '{"task": "text-classification", 
                    "model": "roberta-large-mnli"
                    }'::JSONB
        ) AS nli;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            {"label": "ENTAILMENT", "score": 0.98837411403656}
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_qnli() {
        let sql = r#"SELECT pgml.transform(
            inputs => ARRAY[
                'Where is the capital of France?, Paris is the capital of France.'
            ],
            task => '{"task": "text-classification", 
                    "model": "cross-encoder/qnli-electra-base"
                    }'::JSONB
        ) AS qnli;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            {"label": "LABEL_0", "score": 0.9978110194206238}
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_qqp() {
        let sql = r#"SELECT pgml.transform(
            inputs => ARRAY[
                'Which city is the capital of France?, Where is the capital of France?'
            ],
            task => '{"task": "text-classification", 
                    "model": "textattack/bert-base-uncased-QQP"
                    }'::JSONB
        ) AS qqp;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            {"label": "LABEL_0", "score": 0.9988721013069152}
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_grammatical_correctness() {
        let sql = r#"SELECT pgml.transform(
            inputs => ARRAY[
                'I will walk to home when I went through the bus.'
            ],
            task => '{"task": "text-classification", 
                    "model": "textattack/distilbert-base-uncased-CoLA"
                    }'::JSONB
        ) AS grammatical_correctness;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            {"label": "LABEL_1", "score": 0.9576480388641356}
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_zeroshot_classification() {
        let sql = r#"SELECT pgml.transform(
            inputs => ARRAY[
                'I have a problem with my iphone that needs to be resolved asap!!'
            ],
            task => '{
                        "task": "zero-shot-classification", 
                        "model": "facebook/bart-large-mnli"
                    }'::JSONB,
            args => '{
                        "candidate_labels": ["urgent", "not urgent", "phone", "tablet", "computer"]
                    }'::JSONB
        ) AS zero_shot;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            {
                "labels": ["urgent", "phone", "computer", "not urgent", "tablet"],
                "scores": [0.503635, 0.47879, 0.012600, 0.002655, 0.002308],
                "sequence": "I have a problem with my iphone that needs to be resolved asap!!"
            }
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_token_classification_ner() {
        let sql = r#"SELECT pgml.transform(
            inputs => ARRAY[
                'I am Omar and I live in New York City.'
            ],
            task => 'token-classification'
        ) as ner;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([[
            {"end": 9,  "word": "Omar", "index": 3,  "score": 0.997110, "start": 5,  "entity": "I-PER"},
            {"end": 27, "word": "New",  "index": 8,  "score": 0.999372, "start": 24, "entity": "I-LOC"},
            {"end": 32, "word": "York", "index": 9,  "score": 0.999355, "start": 28, "entity": "I-LOC"},
            {"end": 37, "word": "City", "index": 10, "score": 0.999431, "start": 33, "entity": "I-LOC"}
        ]]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_token_classification_pos() {
        let sql = r#"select pgml.transform(
            inputs => array [
            'I live in Amsterdam.'
            ],
            task => '{"task": "token-classification", 
                    "model": "vblagoje/bert-english-uncased-finetuned-pos"
            }'::JSONB
        ) as pos;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([[
            {"end": 1,  "word": "i",         "index": 1, "score": 0.999, "start": 0,  "entity": "PRON"},
            {"end": 6,  "word": "live",      "index": 2, "score": 0.998, "start": 2,  "entity": "VERB"},
            {"end": 9,  "word": "in",        "index": 3, "score": 0.999, "start": 7,  "entity": "ADP"},
            {"end": 19, "word": "amsterdam", "index": 4, "score": 0.998, "start": 10, "entity": "PROPN"},
            {"end": 20, "word": ".",         "index": 5, "score": 0.999, "start": 19, "entity": "PUNCT"}
        ]]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_translation() {
        let sql = r#"select pgml.transform(
            inputs => array[
                        'How are you?'
            ],
            task => '{"task": "translation", 
                    "model": "Helsinki-NLP/opus-mt-en-fr"
            }'::JSONB	
        );"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            {"translation_text": "Comment allez-vous ?"}
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_summarization() {
        let sql = r#"select pgml.transform(
            task => '{"task": "summarization", 
                    "model": "sshleifer/distilbart-cnn-12-6"
            }'::JSONB,
            inputs => array[
            'Paris is the capital and most populous city of France, with an estimated population of 2,175,601 residents as of 2018, in an area of more than 105 square kilometres (41 square miles). The City of Paris is the centre and seat of government of the region and province of Île-de-France, or Paris Region, which has an estimated population of 12,174,880, or about 18 percent of the population of France as of 2017.'
            ]
        );"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            {"summary_text": " Paris is the capital and most populous city of France, with an estimated population of 2,175,601 residents as of 2018 . The city is the centre and seat of government of the region and province of Île-de-France, or Paris Region . Paris Region has an estimated 18 percent of the population of France as of 2017 ."}
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_summarization_min_max_length() {
        let sql = r#"select pgml.transform(
            task => '{"task": "summarization", 
                    "model": "sshleifer/distilbart-cnn-12-6"
            }'::JSONB,
            inputs => array[
            'Paris is the capital and most populous city of France, with an estimated population of 2,175,601 residents as of 2018, in an area of more than 105 square kilometres (41 square miles). The City of Paris is the centre and seat of government of the region and province of Île-de-France, or Paris Region, which has an estimated population of 12,174,880, or about 18 percent of the population of France as of 2017.'
            ],
            args => '{
                    "min_length" : 20,
                    "max_length" : 70
            }'::JSONB
        );"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            {"summary_text": " Paris is the capital and most populous city of France, with an estimated population of 2,175,601 residents as of 2018 . City of Paris is centre and seat of government of the region and province of Île-de-France, or Paris Region, which has an estimated 12,174,880, or about 18 percent"}
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_question_answering() {
        let sql = r#"SELECT pgml.transform(
            'question-answering',
            inputs => ARRAY[
                '{
                    "question": "Where do I live?",
                    "context": "My name is Merve and I live in İstanbul."
                }'
            ]
        ) AS answer;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!({
            "end"   :  39,
            "score" :  0.9538117051124572,
            "start" :  31,
            "answer": "İstanbul"
        });
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_text_generation() {
        let sql = r#"SELECT pgml.transform(
            task => 'text-generation',
            inputs => ARRAY[
                'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
            ]
        ) AS answer;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            [
                {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, and eight for the Dragon-lords in their halls of blood.\n\nEach of the guild-building systems is one-man"}
            ]
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_text_generation_specific_model() {
        let sql = r#"SELECT pgml.transform(
            task => '{
                "task" : "text-generation",
                "model" : "gpt2-medium"
            }'::JSONB,
            inputs => ARRAY[
                'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
            ]
        ) AS answer;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            [{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone.\n\nThis place has a deep connection to the lore of ancient Elven civilization. It is home to the most ancient of artifacts,"}]
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_text_generation_max_length() {
        let sql = r#"SELECT pgml.transform(
            task => '{
                "task" : "text-generation",
                "model" : "gpt2-medium"
            }'::JSONB,
            inputs => ARRAY[
                'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
            ],
            args => '{
                    "max_length" : 200
                }'::JSONB 
        ) AS answer;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            [{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, Three for the Dwarfs and the Elves, One for the Gnomes of the Mines, and Two for the Elves of Dross.\"\n\nHobbits: The Fellowship is the first book of J.R.R. Tolkien's story-cycle, and began with his second novel - The Two Towers - and ends in The Lord of the Rings.\n\n\nIt is a non-fiction novel, so there is no copyright claim on some parts of the story but the actual text of the book is copyrighted by author J.R.R. Tolkien.\n\n\nThe book has been classified into two types: fantasy novels and children's books\n\nHobbits: The Fellowship is the first book of J.R.R. Tolkien's story-cycle, and began with his second novel - The Two Towers - and ends in The Lord of the Rings.It"}]
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_text_generation_num_return_sequences() {
        let sql = r#"SELECT pgml.transform(
            task => '{
                "task" : "text-generation",
                "model" : "gpt2-medium"
            }'::JSONB,
            inputs => ARRAY[
                'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
            ],
            args => '{
                    "num_return_sequences" : 3
                }'::JSONB 
        ) AS answer;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            [
                {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, and Thirteen for the human-men in their hall of fire.\n\nAll of us, our families, and our people"},
                {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, and the tenth for a King! As each of these has its own special story, so I have written them into the game."},
                {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone… What's left in the end is your heart's desire after all!\n\nHans: (Trying to be brave)"}
            ]
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_text_generation_beams_stopping() {
        let sql = r#"SELECT pgml.transform(
            task => '{
                "task" : "text-generation",
                "model" : "gpt2-medium"
            }'::JSONB,
            inputs => ARRAY[
                'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
            ],
            args => '{
                    "num_beams" : 5,
                    "early_stopping" : true
                }'::JSONB 
        ) AS answer;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([[
            {"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, Nine for the Dwarves in their caverns of ice, Ten for the Elves in their caverns of fire, Eleven for the"}
        ]]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_text_generation_temperature() {
        let sql = r#"SELECT pgml.transform(
            task => '{
                "task" : "text-generation",
                "model" : "gpt2-medium"
            }'::JSONB,
            inputs => ARRAY[
                'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
            ],
            args => '{
                    "do_sample" : true,
                    "temperature" : 0.9
                }'::JSONB 
        ) AS answer;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([[{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, and Thirteen for the Giants and Men of S.A.\n\nThe First Seven-Year Time-Traveling Trilogy is"}]]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_text_generation_top_p() {
        let sql = r#"SELECT pgml.transform(
            task => '{
                "task" : "text-generation",
                "model" : "gpt2-medium"
            }'::JSONB,
            inputs => ARRAY[
                'Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone'
            ],
            args => '{
                    "do_sample" : true,
                    "top_p" : 0.8
                }'::JSONB 
        ) AS answer;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([[{"generated_text": "Three Rings for the Elven-kings under the sky, Seven for the Dwarf-lords in their halls of stone, Four for the Elves of the forests and fields, and Three for the Dwarfs and their warriors.\" ―Lord Rohan [src"}]]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_text_text_generation() {
        let sql = r#"SELECT pgml.transform(
            task => '{
                "task" : "text2text-generation"
            }'::JSONB,
            inputs => ARRAY[
                'translate from English to French: I''m very happy'
            ]
        ) AS answer;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            {"generated_text": "Je suis très heureux"}
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn readme_nlp_fill_mask() {
        let sql = r#"SELECT pgml.transform(
            task => '{
                "task" : "fill-mask"
            }'::JSONB,
            inputs => ARRAY[
                'Paris is the <mask> of France.'

            ]
        ) AS answer;"#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([
            {"score": 0.679, "token": 812,   "sequence": "Paris is the capital of France.",    "token_str": " capital"},
            {"score": 0.051, "token": 32357, "sequence": "Paris is the birthplace of France.", "token_str": " birthplace"},
            {"score": 0.038, "token": 1144,  "sequence": "Paris is the heart of France.",      "token_str": " heart"},
            {"score": 0.024, "token": 29778, "sequence": "Paris is the envy of France.",       "token_str": " envy"},
            {"score": 0.022, "token": 1867,  "sequence": "Paris is the Capital of France.",    "token_str": " Capital"}
        ]);
        assert_eq!(got, want);
    }

    #[pg_test]
    #[ignore = "requires model download"]
    fn template() {
        let sql = r#""#;
        let got = Spi::get_one::<JsonB>(sql).unwrap().unwrap().0;
        let want = serde_json::json!([]);
        assert_eq!(got, want);
    }

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
            Some(vec!["target".to_string()]),
            0.5,
            Sampling::last,
            true,
            JsonB(serde_json::Value::Object(Hyperparams::new())),
        );
        assert!(snapshot.id > 0);
    }

    #[pg_test]
    fn test_not_fully_qualified_table() {
        load_diabetes(Some(25));

        let result = std::panic::catch_unwind(|| {
            let _snapshot = Snapshot::create(
                "diabetes",
                Some(vec!["target".to_string()]),
                0.5,
                Sampling::last,
                true,
                JsonB(serde_json::Value::Object(Hyperparams::new())),
            );
        });

        assert!(result.is_err());
    }

    #[pg_test]
    fn test_train_regression() {
        load_diabetes(None);

        // Modify postgresql.conf and add shared_preload_libraries = 'pgml'
        // to test deployments.
        let setting = Spi::get_one::<String>("select setting from pg_settings where name = 'data_directory'").unwrap();

        notice!("Data directory: {}", setting.unwrap());

        for runtime in [Runtime::python, Runtime::rust] {
            let result: Vec<(String, String, String, bool)> = train(
                "Test project",
                Some(&Task::regression.to_string()),
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
                JsonB(serde_json::Value::Object(Hyperparams::new())),
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
        let setting = Spi::get_one::<String>("select setting from pg_settings where name = 'data_directory'").unwrap();

        notice!("Data directory: {}", setting.unwrap());

        for runtime in [Runtime::python, Runtime::rust] {
            let result: Vec<(String, String, String, bool)> = train(
                "Test project 2",
                Some(&Task::classification.to_string()),
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
                JsonB(serde_json::Value::Object(Hyperparams::new())),
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
        let setting = Spi::get_one::<String>("select setting from pg_settings where name = 'data_directory'").unwrap();

        notice!("Data directory: {}", setting.unwrap());

        for runtime in [Runtime::python, Runtime::rust] {
            let result: Vec<(String, String, String, bool)> = train(
                "Test project 3",
                Some(&Task::classification.to_string()),
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
                JsonB(serde_json::Value::Object(Hyperparams::new())),
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
