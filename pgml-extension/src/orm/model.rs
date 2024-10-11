use anyhow::{anyhow, bail, Result};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::num::NonZeroUsize;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use ::linfa::prelude::{BinaryClassification, Pr, SingleTargetRegression, ToConfusionMatrix};
use indexmap::IndexMap;
use itertools::{izip, Itertools};
use ndarray::ArrayView1;
use once_cell::sync::Lazy;
use pgrx::heap_tuple::PgHeapTuple;
use pgrx::*;
use rand::prelude::SliceRandom;
use serde_json::json;

use crate::bindings::*;
use crate::orm::*;

#[allow(clippy::type_complexity)]
static DEPLOYED_MODELS_BY_ID: Lazy<Mutex<HashMap<i64, Arc<Model>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
pub struct Model {
    pub id: i64,
    pub project_id: i64,
    pub snapshot_id: i64,
    pub algorithm: Algorithm,
    pub hyperparams: JsonB,
    pub runtime: Runtime,
    pub status: Status,
    pub metrics: Option<JsonB>,
    pub search: Option<Search>,
    pub search_params: JsonB,
    pub search_args: JsonB,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub project: Project,
    pub snapshot: Snapshot,
    pub bindings: Option<Box<dyn Bindings>>,
    pub num_classes: usize,
    pub num_features: usize,
}

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "Model {{ id: {}, task: {:?}, algorithm: {:?}, runtime: {:?} }}",
            self.id, self.project.task, self.algorithm, self.runtime
        )
    }
}

impl Model {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        project: &Project,
        snapshot: &mut Snapshot,
        algorithm: Algorithm,
        hyperparams: JsonB,
        search: Option<Search>,
        search_params: JsonB,
        search_args: JsonB,
        runtime: Option<Runtime>,
    ) -> Model {
        let mut model: Option<Model> = None;

        // Set the runtime to one we recommend, unless the user knows better.
        let runtime = match runtime {
            Some(runtime) => runtime,
            None => match snapshot.y_column_name.len() {
                1 => match algorithm {
                    Algorithm::xgboost => Runtime::rust,
                    Algorithm::lightgbm => Runtime::rust,
                    Algorithm::linear => match project.task {
                        Task::classification => Runtime::python,
                        Task::regression => Runtime::rust,
                        _ => error!("No default runtime available for tasks other than `classification` and `regression` when using a linear algorithm."),
                    },
                    _ => Runtime::python,
                },
                // Joint regression is only supported in Python
                _ => Runtime::python,
            },
        };

        let dataset = snapshot.tabular_dataset();
        let status = Status::in_progress;
        // Create the model record.
        Spi::connect(|mut client| {
            let result = client.update("
          INSERT INTO pgml.models (project_id, snapshot_id, algorithm, runtime, hyperparams, status, search, search_params, search_args, num_features)
          VALUES ($1, $2, cast($3 AS pgml.algorithm), cast($4 AS pgml.runtime), $5, cast($6 as pgml.status), $7, $8, $9, $10)
          RETURNING id, project_id, snapshot_id, algorithm, runtime, hyperparams, status, metrics, search, search_params, search_args, created_at, updated_at;",
                                       Some(1),
                                       Some(vec![
                                           (PgBuiltInOids::INT8OID.oid(), project.id.into_datum()),
                                           (PgBuiltInOids::INT8OID.oid(), snapshot.id.into_datum()),
                                           (PgBuiltInOids::TEXTOID.oid(), algorithm.to_string().into_datum()),
                                           (PgBuiltInOids::TEXTOID.oid(), runtime.to_string().into_datum()),
                                           (PgBuiltInOids::JSONBOID.oid(), hyperparams.into_datum()),
                                           (PgBuiltInOids::TEXTOID.oid(), status.to_string().into_datum()),
                                           (PgBuiltInOids::TEXTOID.oid(), search.map(|search| search.to_string()).into_datum()),
                                           (PgBuiltInOids::JSONBOID.oid(), search_params.into_datum()),
                                           (PgBuiltInOids::JSONBOID.oid(), search_args.into_datum()),
                                           (PgBuiltInOids::INT8OID.oid(), (dataset.num_features as i64).into_datum()),
                                       ]),
            ).unwrap().first();
            if !result.is_empty() {
                model = Some(Model {
                    id: result.get(1).unwrap().unwrap(),
                    project_id: result.get(2).unwrap().unwrap(),
                    snapshot_id: result.get(3).unwrap().unwrap(),
                    algorithm, // 4
                    runtime,   // 5
                    hyperparams: result.get(6).unwrap().unwrap(),
                    status, // 6,
                    metrics: result.get(8).unwrap(),
                    search, // 9
                    search_params: result.get(10).unwrap().unwrap(),
                    search_args: result.get(11).unwrap().unwrap(),
                    created_at: result.get(12).unwrap().unwrap(),
                    updated_at: result.get(13).unwrap().unwrap(),
                    project: project.clone(),
                    snapshot: snapshot.clone(),
                    bindings: None,
                    num_classes: match project.task {
                        Task::regression => 0,
                        _ => snapshot.num_classes(),
                    },
                    num_features: snapshot.num_features(),
                });
            }
        });

        let mut model = model.unwrap();

        info!("Training {}", model);
        model.fit(&dataset);

        Spi::run_with_args(
            "UPDATE pgml.models SET status = $1::pgml.status WHERE id = $2",
            Some(vec![
                (
                    PgBuiltInOids::TEXTOID.oid(),
                    Status::successful.to_string().into_datum(),
                ),
                (PgBuiltInOids::INT8OID.oid(), model.id.into_datum()),
            ]),
        )
        .unwrap();

        model
    }

    #[allow(clippy::too_many_arguments)]
    pub fn finetune(project: &Project, snapshot: &mut Snapshot, hyperparams: &JsonB) -> Model {
        let mut model: Option<Model> = None;

        let dataset_args = JsonB(json!(hyperparams.0.get("dataset_args").unwrap()));

        // let dataset = snapshot.text_classification_dataset(dataset_args);
        let dataset = if project.task == Task::text_classification {
            TextDatasetType::TextClassification(snapshot.text_classification_dataset(dataset_args))
        } else if project.task == Task::text_pair_classification {
            TextDatasetType::TextPairClassification(snapshot.text_pair_classification_dataset(dataset_args))
        } else if project.task == Task::conversation {
            TextDatasetType::Conversation(snapshot.conversation_dataset(dataset_args))
        } else {
            panic!("Unsupported task for finetuning")
        };

        // Create the model record.
        Spi::connect(|mut client| {
            let result = client.update("
          INSERT INTO pgml.models (project_id, snapshot_id, algorithm, runtime, hyperparams, status, search, search_params, search_args, num_features)
          VALUES ($1, $2, cast($3 AS pgml.algorithm), cast($4 AS pgml.runtime), $5, cast($6 as pgml.status), $7, $8, $9, $10)
          RETURNING id, project_id, snapshot_id, algorithm, runtime::TEXT, hyperparams, status, metrics, search, search_params, search_args, created_at, updated_at;",
                                       Some(1),
                                       Some(vec![
                                           (PgBuiltInOids::INT8OID.oid(), project.id.into_datum()),
                                           (PgBuiltInOids::INT8OID.oid(), snapshot.id.into_datum()),
                                           (PgBuiltInOids::TEXTOID.oid(), Algorithm::transformers.to_string().into_datum()),
                                           (PgBuiltInOids::TEXTOID.oid(), Runtime::python.to_string().into_datum()),
                                           (PgBuiltInOids::JSONBOID.oid(), JsonB(json!(hyperparams)).into_datum()),
                                           (PgBuiltInOids::TEXTOID.oid(), Status::in_progress.to_string().into_datum()),
                                           (PgBuiltInOids::TEXTOID.oid(), None::<Option<Search>>.into_datum()),
                                           (PgBuiltInOids::JSONBOID.oid(), JsonB(serde_json::from_str("{}").unwrap()).into_datum()),
                                           (PgBuiltInOids::JSONBOID.oid(), JsonB(serde_json::from_str("{}").unwrap()).into_datum()),
                                           (PgBuiltInOids::INT8OID.oid(), (dataset.num_features() as i64).into_datum()),
                                       ]),
            ).unwrap().first();
            if !result.is_empty() {
                model = Some(Model {
                    id: result.get(1).unwrap().unwrap(),
                    project_id: result.get(2).unwrap().unwrap(),
                    snapshot_id: result.get(3).unwrap().unwrap(),
                    algorithm: Algorithm::from_str(result.get(4).unwrap().unwrap()).unwrap(),
                    runtime: Runtime::from_str(result.get(5).unwrap().unwrap()).unwrap(),
                    hyperparams: result.get(6).unwrap().unwrap(),
                    status: Status::from_str(result.get(7).unwrap().unwrap()).unwrap(),
                    metrics: result.get(8).unwrap(),
                    search: result.get(9).unwrap().map(|search| Search::from_str(search).unwrap()),
                    search_params: result.get(10).unwrap().unwrap(),
                    search_args: result.get(11).unwrap().unwrap(),
                    created_at: result.get(12).unwrap().unwrap(),
                    updated_at: result.get(13).unwrap().unwrap(),
                    project: project.clone(),
                    snapshot: snapshot.clone(),
                    bindings: None,
                    num_classes: 0,
                    num_features: snapshot.num_features(),
                });
            }
        });

        let mut model = model.unwrap();
        let id = model.id;
        let path = std::path::PathBuf::from(format!("/tmp/postgresml/models/{id}"));

        info!("Tuning {}", model);
        let metrics: HashMap<String, f64>;
        match dataset {
            TextDatasetType::TextClassification(dataset) => {
                metrics = match transformers::finetune_text_classification(
                    &project.task,
                    dataset,
                    &model.hyperparams,
                    &path,
                    project.id,
                    model.id,
                ) {
                    Ok(metrics) => metrics,
                    Err(e) => error!("{e}"),
                };
            }
            TextDatasetType::TextPairClassification(dataset) => {
                metrics = match transformers::finetune_text_pair_classification(
                    &project.task,
                    dataset,
                    &model.hyperparams,
                    &path,
                    project.id,
                    model.id,
                ) {
                    Ok(metrics) => metrics,
                    Err(e) => error!("{e}"),
                };
            }
            TextDatasetType::Conversation(dataset) => {
                metrics = match transformers::finetune_conversation(
                    &project.task,
                    dataset,
                    &model.hyperparams,
                    &path,
                    project.id,
                    model.id,
                ) {
                    Ok(metrics) => metrics,
                    Err(e) => error!("{e}"),
                };
            }
        };

        model.metrics = Some(JsonB(json!(metrics)));
        info!("Metrics: {:?}", &metrics);

        Spi::get_one_with_args::<i64>(
            "UPDATE pgml.models SET hyperparams = $1, metrics = $2 WHERE id = $3 RETURNING id",
            vec![
                (
                    PgBuiltInOids::JSONBOID.oid(),
                    JsonB(model.hyperparams.0.clone()).into_datum(),
                ),
                (
                    PgBuiltInOids::JSONBOID.oid(),
                    JsonB(model.metrics.as_ref().unwrap().0.clone()).into_datum(),
                ),
                (PgBuiltInOids::INT8OID.oid(), model.id.into_datum()),
            ],
        )
        .unwrap();

        // Save the bindings.
        if path.is_dir() {
            for entry in std::fs::read_dir(&path).unwrap() {
                let path = entry.unwrap().path();

                if path.is_file() {
                    let bytes = std::fs::read(&path).unwrap();

                    for (i, chunk) in bytes.chunks(100_000_000).enumerate() {
                        Spi::get_one_with_args::<i64>(
                            "INSERT INTO pgml.files (model_id, path, part, data) VALUES($1, $2, $3, $4) RETURNING id",
                            vec![
                                (PgBuiltInOids::INT8OID.oid(), model.id.into_datum()),
                                (
                                    PgBuiltInOids::TEXTOID.oid(),
                                    path.file_name().unwrap().to_str().into_datum(),
                                ),
                                (PgBuiltInOids::INT8OID.oid(), (i as i64).into_datum()),
                                (PgBuiltInOids::BYTEAOID.oid(), chunk.into_datum()),
                            ],
                        )
                        .unwrap();
                    }
                }
            }
        } else {
            error!("Model checkpoint folder does not exist!")
        }

        Spi::run_with_args(
            "UPDATE pgml.models SET status = $1::pgml.status WHERE id = $2",
            Some(vec![
                (
                    PgBuiltInOids::TEXTOID.oid(),
                    Status::successful.to_string().into_datum(),
                ),
                (PgBuiltInOids::INT8OID.oid(), model.id.into_datum()),
            ]),
        )
        .unwrap();

        model
    }

    fn find(id: i64) -> Result<Model> {
        let mut model = None;
        // Create the model record.
        Spi::connect(|client| {
            let result = client.select("
                SELECT id, project_id, snapshot_id, algorithm::TEXT, runtime::TEXT, hyperparams, status, metrics, search::TEXT, search_params, search_args, created_at, updated_at
                FROM pgml.models
                WHERE id = $1;",
                                       Some(1),
                                       Some(vec![
                                           (PgBuiltInOids::INT8OID.oid(), id.into_datum()),
                                       ]),
            ).unwrap().first();

            if !result.is_empty() {
                let project_id = result.get(2).unwrap().expect("project_id is i64");
                let project = Project::find(project_id).expect("project doesn't exist");
                let snapshot_id = result.get(3).unwrap().expect("snapshot_id is i64");
                let snapshot = Snapshot::find(snapshot_id).expect("snapshot doesn't exist");
                let algorithm = Algorithm::from_str(result.get(4).unwrap().unwrap()).expect("algorithm is malformed");
                let runtime = Runtime::from_str(result.get(5).unwrap().unwrap()).expect("runtime is malformed");
                let data = Spi::get_one_with_args::<Vec<u8>>(
                    "
                        SELECT data
                        FROM pgml.files
                        WHERE files.model_id = $1
                        LIMIT 1",
                    vec![(PgBuiltInOids::INT8OID.oid(), id.into_datum())],
                )
                .unwrap()
                .unwrap();
                let hyperparams = result.get(11).unwrap().unwrap();

                let bindings: Box<dyn Bindings> = match runtime {
                    Runtime::openai => {
                        error!("OpenAI runtime is not supported for training or inference");
                    }

                    Runtime::rust => {
                        match algorithm {
                            Algorithm::xgboost => {
                                xgboost::Estimator::from_bytes(&data, &hyperparams)?
                            }
                            Algorithm::lightgbm => {
                                lightgbm::Estimator::from_bytes(&data, &hyperparams)?
                            }
                            Algorithm::linear => match project.task {
                                Task::regression => {
                                    linfa::LinearRegression::from_bytes(&data, &hyperparams)?
                                }
                                Task::classification => {
                                    linfa::LogisticRegression::from_bytes(&data, &hyperparams)?
                                }
                                _ => bail!("No default runtime available for tasks other than `classification` and `regression` when using a linear algorithm."),
                            },
                            Algorithm::svm => linfa::Svm::from_bytes(&data, &hyperparams)?,
                            _ => todo!(), //smartcore_load(&data, task, algorithm, &hyperparams),
                        }
                    }

                    #[cfg(feature = "python")]
                    Runtime::python => sklearn::Estimator::from_bytes(&data, &hyperparams)?,

                    #[cfg(not(feature = "python"))]
                    Runtime::python => {
                        bail!("Python runtime not supported, recompile with `--features python`")
                    }
                };

                let num_features = snapshot.num_features();
                let num_classes = match project.task {
                    Task::regression => 0,
                    _ => snapshot.num_classes(),
                };

                model = Some(Model {
                    id: result.get(1).unwrap().unwrap(),
                    project_id,
                    snapshot_id,
                    algorithm,
                    runtime,
                    hyperparams: hyperparams,
                    status: Status::from_str(result.get(7).unwrap().unwrap()).unwrap(),
                    metrics: result.get(8).unwrap(),
                    search: result.get(9).unwrap().map(|search| Search::from_str(search).unwrap()),
                    search_params: result.get(10).unwrap().unwrap(),
                    search_args: result.get(11).unwrap().unwrap(),
                    created_at: result.get(12).unwrap().unwrap(),
                    updated_at: result.get(13).unwrap().unwrap(),
                    project,
                    snapshot,
                    bindings: Some(bindings),
                    num_classes,
                    num_features,
                });
            }

            Ok(())
        })?;

        model.ok_or_else(|| anyhow!("pgml.models WHERE id = {:?} could not be loaded. Does it exist?", id))
    }

    pub fn find_cached(id: i64) -> Result<Arc<Model>> {
        {
            let models = DEPLOYED_MODELS_BY_ID.lock();
            if let Some(model) = models.get(&id) {
                return Ok(model.clone());
            }
        }

        info!("Model cache miss {:?}", id);
        let model = Arc::new(Model::find(id)?);
        let mut models = DEPLOYED_MODELS_BY_ID.lock();
        models.insert(id, Arc::clone(&model));
        Ok(model)
    }

    fn get_fit_function(&self) -> crate::bindings::Fit {
        match self.runtime {
            Runtime::openai => {
                error!("OpenAI runtime is not supported for training or inference");
            }

            Runtime::rust => match self.project.task {
                Task::regression => match self.algorithm {
                    Algorithm::xgboost => xgboost::fit_regression,
                    Algorithm::lightgbm => lightgbm::fit_regression,
                    Algorithm::linear => linfa::LinearRegression::fit,
                    Algorithm::svm => linfa::Svm::fit,
                    _ => todo!(),
                },
                Task::classification => match self.algorithm {
                    Algorithm::xgboost => xgboost::fit_classification,
                    Algorithm::lightgbm => lightgbm::fit_classification,
                    Algorithm::linear => linfa::LogisticRegression::fit,
                    Algorithm::svm => linfa::Svm::fit,
                    _ => todo!(),
                },
                Task::decomposition => todo!(),
                Task::clustering => todo!(),
                _ => error!("use pgml.tune for transformers tasks"),
            },

            #[cfg(not(feature = "python"))]
            Runtime::python => {
                error!("Python runtime not supported, recompile with `--features python`")
            }

            #[cfg(feature = "python")]
            Runtime::python => match self.project.task {
                Task::regression => match self.algorithm {
                    Algorithm::linear => sklearn::linear_regression,
                    Algorithm::lasso => sklearn::lasso_regression,
                    Algorithm::svm => sklearn::svm_regression,
                    Algorithm::elastic_net => sklearn::elastic_net_regression,
                    Algorithm::ridge => sklearn::ridge_regression,
                    Algorithm::random_forest => sklearn::random_forest_regression,
                    Algorithm::xgboost => sklearn::xgboost_regression,
                    Algorithm::xgboost_random_forest => sklearn::xgboost_random_forest_regression,
                    Algorithm::orthogonal_matching_pursuit => sklearn::orthogonal_matching_pursuit_regression,
                    Algorithm::bayesian_ridge => sklearn::bayesian_ridge_regression,
                    Algorithm::automatic_relevance_determination => {
                        sklearn::automatic_relevance_determination_regression
                    }
                    Algorithm::stochastic_gradient_descent => sklearn::stochastic_gradient_descent_regression,
                    Algorithm::passive_aggressive => sklearn::passive_aggressive_regression,
                    Algorithm::ransac => sklearn::ransac_regression,
                    Algorithm::theil_sen => sklearn::theil_sen_regression,
                    Algorithm::huber => sklearn::huber_regression,
                    Algorithm::quantile => sklearn::quantile_regression,
                    Algorithm::kernel_ridge => sklearn::kernel_ridge_regression,
                    Algorithm::gaussian_process => sklearn::gaussian_process_regression,
                    Algorithm::nu_svm => sklearn::nu_svm_regression,
                    Algorithm::ada_boost => sklearn::ada_boost_regression,
                    Algorithm::bagging => sklearn::bagging_regression,
                    Algorithm::extra_trees => sklearn::extra_trees_regression,
                    Algorithm::gradient_boosting_trees => sklearn::gradient_boosting_trees_regression,
                    Algorithm::hist_gradient_boosting => sklearn::hist_gradient_boosting_regression,
                    Algorithm::least_angle => sklearn::least_angle_regression,
                    Algorithm::lasso_least_angle => sklearn::lasso_least_angle_regression,
                    Algorithm::linear_svm => sklearn::linear_svm_regression,
                    Algorithm::lightgbm => sklearn::lightgbm_regression,
                    Algorithm::catboost => sklearn::catboost_regression,
                    _ => error!("{:?} does not support regression", self.algorithm),
                },
                Task::classification => match self.algorithm {
                    Algorithm::linear => sklearn::linear_classification,
                    Algorithm::svm => sklearn::svm_classification,
                    Algorithm::ridge => sklearn::ridge_classification,
                    Algorithm::random_forest => sklearn::random_forest_classification,
                    Algorithm::xgboost => sklearn::xgboost_classification,
                    Algorithm::xgboost_random_forest => sklearn::xgboost_random_forest_classification,
                    Algorithm::stochastic_gradient_descent => sklearn::stochastic_gradient_descent_classification,
                    Algorithm::perceptron => sklearn::perceptron_classification,
                    Algorithm::passive_aggressive => sklearn::passive_aggressive_classification,
                    Algorithm::gaussian_process => sklearn::gaussian_process,
                    Algorithm::nu_svm => sklearn::nu_svm_classification,
                    Algorithm::ada_boost => sklearn::ada_boost_classification,
                    Algorithm::bagging => sklearn::bagging_classification,
                    Algorithm::extra_trees => sklearn::extra_trees_classification,
                    Algorithm::gradient_boosting_trees => sklearn::gradient_boosting_trees_classification,
                    Algorithm::hist_gradient_boosting => sklearn::hist_gradient_boosting_classification,
                    Algorithm::linear_svm => sklearn::linear_svm_classification,
                    Algorithm::lightgbm => sklearn::lightgbm_classification,
                    Algorithm::catboost => sklearn::catboost_classification,
                    _ => error!("{:?} does not support classification", self.algorithm),
                },
                Task::clustering => match self.algorithm {
                    Algorithm::affinity_propagation => sklearn::affinity_propagation,
                    Algorithm::birch => sklearn::birch,
                    Algorithm::kmeans => sklearn::kmeans,
                    Algorithm::mini_batch_kmeans => sklearn::mini_batch_kmeans,
                    Algorithm::mean_shift => sklearn::mean_shift,
                    _ => error!("{:?} does not support clustering", self.algorithm),
                },
                Task::decomposition => match self.algorithm {
                    Algorithm::pca => sklearn::pca,
                    _ => error!("{:?} does not support clustering", self.algorithm),
                },
                _ => error!("use pgml.tune for transformers tasks"),
            },
        }
    }

    /// Generates a complete list of hyperparams that should be tested
    /// by combining the self.search_params. When search params are empty,
    /// the set only contains the self.hyperparams.
    fn get_all_hyperparams(&self, n_iter: usize) -> Vec<Hyperparams> {
        // Gather all hyperparams
        let mut all_hyperparam_names = Vec::new();
        let mut all_hyperparam_values = Vec::new();
        for (key, value) in self.hyperparams.0.as_object().unwrap() {
            all_hyperparam_names.push(key.to_string());
            all_hyperparam_values.push(vec![value.clone()]);
        }
        for (key, values) in self.search_params.0.as_object().unwrap() {
            if all_hyperparam_names.contains(key) {
                error!(
                    "`{key}` cannot be present in both hyperparams and search_params. Please choose one or the other."
                );
            }
            all_hyperparam_names.push(key.to_string());
            all_hyperparam_values.push(values.as_array().unwrap().to_vec());
        }

        // The search space is all possible combinations
        let all_hyperparam_values: Vec<Vec<serde_json::Value>> =
            all_hyperparam_values.into_iter().multi_cartesian_product().collect();
        let mut all_hyperparam_values = match self.search {
            Some(Search::random) => {
                // TODO support things like ranges to be random sampled
                let mut rng = &mut rand::thread_rng();
                all_hyperparam_values
                    .choose_multiple(&mut rng, n_iter)
                    .cloned()
                    .collect()
            }
            _ => all_hyperparam_values,
        };

        // Empty set for a run of only the default values
        if all_hyperparam_values.is_empty() {
            all_hyperparam_values.push(Vec::new());
        }

        // Construct sets of hyperparams from the values
        all_hyperparam_values
            .iter()
            .map(|hyperparam_values| {
                let mut hyperparams = Hyperparams::new();
                for (idx, value) in hyperparam_values.iter().enumerate() {
                    let name = all_hyperparam_names[idx].clone();
                    hyperparams.insert(name, value.clone());
                }
                hyperparams
            })
            .collect()
    }

    // The box is borrowed so that it may be reused by the caller
    #[allow(clippy::borrowed_box)]
    fn test(&self, dataset: &Dataset) -> IndexMap<String, f32> {
        info!("Testing {:?} estimator {:?}", self.project.task, self);
        // Test the estimator on the data
        let y_hat = self.predict_batch(&dataset.x_test).unwrap();
        let y_test = &dataset.y_test;

        // Calculate metrics to evaluate this estimator and its hyperparams
        let mut metrics = IndexMap::new();
        match self.project.task {
            Task::regression => {
                #[cfg(all(feature = "python", any(test, feature = "pg_test")))]
                {
                    let sklearn_metrics = sklearn::regression_metrics(y_test, &y_hat).unwrap();
                    metrics.insert("sklearn_r2".to_string(), sklearn_metrics["r2"]);
                    metrics.insert("sklearn_mean_absolute_error".to_string(), sklearn_metrics["mae"]);
                    metrics.insert("sklearn_mean_squared_error".to_string(), sklearn_metrics["mse"]);
                }

                let y_test = ArrayView1::from(&y_test);
                let y_hat = ArrayView1::from(&y_hat);

                metrics.insert("r2".to_string(), y_hat.r2(&y_test).unwrap());
                metrics.insert(
                    "mean_absolute_error".to_string(),
                    y_hat.mean_absolute_error(&y_test).unwrap(),
                );
                metrics.insert(
                    "mean_squared_error".to_string(),
                    y_hat.mean_squared_error(&y_test).unwrap(),
                );
            }
            Task::classification => {
                #[cfg(all(feature = "python", any(test, feature = "pg_test")))]
                {
                    let sklearn_metrics =
                        sklearn::classification_metrics(y_test, &y_hat, dataset.num_distinct_labels).unwrap();

                    if dataset.num_distinct_labels == 2 {
                        metrics.insert("sklearn_roc_auc".to_string(), sklearn_metrics["roc_auc"]);
                    }

                    metrics.insert("sklearn_f1".to_string(), sklearn_metrics["f1"]);
                    metrics.insert("sklearn_f1_micro".to_string(), sklearn_metrics["f1_micro"]);
                    metrics.insert("sklearn_precision".to_string(), sklearn_metrics["precision"]);
                    metrics.insert("sklearn_recall".to_string(), sklearn_metrics["recall"]);
                    metrics.insert("sklearn_accuracy".to_string(), sklearn_metrics["accuracy"]);
                    metrics.insert("sklearn_mcc".to_string(), sklearn_metrics["mcc"]);

                    // You can always compare Scikit's confusion matrix to ours
                    // for debugging.
                    // let _sklearn_conf = crate::bindings::sklearn::confusion_matrix(&y_test, &y_hat);
                }

                if dataset.num_distinct_labels == 2 {
                    let y_hat = ArrayView1::from(&y_hat).mapv(Pr::new);
                    let y_test: Vec<bool> = y_test.iter().map(|&i| i == 1.).collect();

                    metrics.insert("roc_auc".to_string(), y_hat.roc(&y_test).unwrap().area_under_curve());
                    metrics.insert("log_loss".to_string(), y_hat.log_loss(&y_test).unwrap());
                }

                let y_hat: Vec<usize> = y_hat.into_iter().map(|i| i.round() as usize).collect();
                let y_test: Vec<usize> = y_test.iter().map(|i| i.round() as usize).collect();
                let y_hat = ArrayView1::from(&y_hat);
                let y_test = ArrayView1::from(&y_test);

                // This one is buggy (Linfa).
                let confusion_matrix = y_hat.confusion_matrix(y_test).unwrap();

                // This has to be identical to Scikit.
                let pgml_confusion_matrix =
                    crate::metrics::ConfusionMatrix::new(&y_test, &y_hat, dataset.num_distinct_labels);

                // These are validated against Scikit and seem to be correct.
                metrics.insert(
                    "f1".to_string(),
                    pgml_confusion_matrix.f1(crate::metrics::Average::Macro),
                );
                metrics.insert("precision".to_string(), pgml_confusion_matrix.precision());
                metrics.insert("recall".to_string(), pgml_confusion_matrix.recall());
                metrics.insert("accuracy".to_string(), pgml_confusion_matrix.accuracy());

                // This one is inaccurate, I have it in my TODO to reimplement.
                metrics.insert("mcc".to_string(), confusion_matrix.mcc());
            }
            Task::clustering => {
                #[cfg(feature = "python")]
                {
                    let sklearn_metrics =
                        sklearn::clustering_metrics(dataset.num_features, &dataset.x_test, &y_hat).unwrap();
                    metrics.insert("silhouette".to_string(), sklearn_metrics["silhouette"]);
                }
            }
            Task::decomposition => {
                #[cfg(feature = "python")]
                {
                    let sklearn_metrics = sklearn::decomposition_metrics(self.bindings.as_ref().unwrap()).unwrap();
                    metrics.insert(
                        "cumulative_explained_variance".to_string(),
                        sklearn_metrics["cumulative_explained_variance"],
                    );
                }
            }
            task => error!("No test metrics available for task: {:?}", task),
        }

        metrics
    }

    fn get_bindings_and_metrics(
        &mut self,
        dataset: &Dataset,
        hyperparams: &Hyperparams,
    ) -> (Box<dyn Bindings>, IndexMap<String, f32>) {
        info!("Hyperparams: {}", serde_json::to_string_pretty(hyperparams).unwrap());

        let fit = self.get_fit_function();
        let now = Instant::now();
        self.bindings = Some(fit(dataset, hyperparams).unwrap());
        let fit_time = now.elapsed();

        let now = Instant::now();
        let mut metrics = self.test(dataset);
        let score_time = now.elapsed();

        metrics.insert("fit_time".to_string(), fit_time.as_secs_f32());
        metrics.insert("score_time".to_string(), score_time.as_secs_f32());
        info!("Metrics: {:?}", &metrics);

        let mut bindings = None;
        std::mem::swap(&mut self.bindings, &mut bindings);
        (bindings.unwrap(), metrics)
    }

    pub fn fit_time(&self) -> f32 {
        self.metrics
            .as_ref()
            .unwrap()
            .0
            .get("fit_time")
            .unwrap()
            .as_f64()
            .unwrap() as f32
    }

    pub fn score_time(&self) -> f32 {
        self.metrics
            .as_ref()
            .unwrap()
            .0
            .get("score_time")
            .unwrap()
            .as_f64()
            .unwrap() as f32
    }

    pub fn f1(&self) -> f32 {
        self.metrics.as_ref().unwrap().0.get("f1").unwrap().as_f64().unwrap() as f32
    }

    pub fn r2(&self) -> f32 {
        self.metrics.as_ref().unwrap().0.get("r2").unwrap().as_f64().unwrap() as f32
    }

    fn fit(&mut self, dataset: &Dataset) {
        // Sometimes our algorithms take a long time. The only way to stop code
        // that we don't have control over is using a signal handler. Signal handlers
        // however are not allowed to allocate any memory. Therefore, we cannot register
        // a SIGINT query cancellation signal and return the connection to a healthy state
        // safely. The only way to cancel a training job then is to send a SIGTERM with
        // `SELECT pg_terminate_backend(pid)` which will process the interrupt, clean up,
        // and close the connection without affecting the postmaster.
        let signal_id = unsafe {
            signal_hook::low_level::register(signal_hook::consts::SIGTERM, || {
                // There can be no memory allocations here.
                check_for_interrupts!();
            })
        }
        .unwrap();

        let mut n_iter: usize = 10;
        let mut cv: usize = if self.search.is_some() { 5 } else { 1 };
        for (key, value) in self.search_args.0.as_object().unwrap() {
            match key.as_str() {
                "n_iter" => n_iter = value.as_i64().unwrap().try_into().unwrap(),
                "cv" => cv = value.as_i64().unwrap().try_into().unwrap(),
                _ => error!("Unknown search_args => {:?}: {:?}", key, value),
            }
        }

        let mut all_hyperparams = self.get_all_hyperparams(n_iter);
        let mut all_bindings = Vec::with_capacity(all_hyperparams.len());
        let mut all_metrics = Vec::with_capacity(all_hyperparams.len());

        info!(
            "Hyperparameter searches: {}, cross validation folds: {}",
            all_hyperparams.len(),
            cv
        );

        // Train and score all the hyperparams on the dataset
        if cv < 2 {
            for hyperparams in &all_hyperparams {
                let (bindings, metrics) = self.get_bindings_and_metrics(dataset, hyperparams);
                all_bindings.push(bindings);
                all_metrics.push(metrics);
            }
        } else {
            // With 2 or more folds, generated for cross validation
            for k in 0..cv {
                let fold = dataset.fold(k, cv);
                for hyperparams in &all_hyperparams {
                    let (bindings, metrics) = self.get_bindings_and_metrics(&fold, hyperparams);
                    all_bindings.push(bindings);
                    all_metrics.push(metrics);
                }
            }
        }

        // Phew, we're done.
        signal_hook::low_level::unregister(signal_id);

        if all_metrics.len() == 1 {
            self.bindings = Some(all_bindings.pop().unwrap());
            self.hyperparams = JsonB(json!(all_hyperparams.pop().unwrap()));
            self.metrics = Some(JsonB(json!(all_metrics.pop().unwrap())));
        } else {
            let mut search_results = IndexMap::new();
            search_results.insert("params".to_string(), json!(all_hyperparams));
            search_results.insert("n_splits".to_string(), json!(cv));

            // Find the best estimator, hyperparams and metrics
            let target_metric = self.project.task.default_target_metric();
            let mut i = 0;
            let mut best_index = 0;
            let mut best_metric = f32::NEG_INFINITY;
            let mut best_metrics = None;
            let mut best_hyperparams = None;
            let mut best_estimator = None;
            let mut fit_times: Vec<Vec<f32>> = vec![vec![0.; cv]; all_hyperparams.len()];
            let mut score_times: Vec<Vec<f32>> = vec![vec![0.; cv]; all_hyperparams.len()];
            let mut test_scores: Vec<Vec<f32>> = vec![vec![0.; cv]; all_hyperparams.len()];
            let mut fold_scores: Vec<Vec<f32>> = vec![vec![0.; all_hyperparams.len()]; cv];
            #[allow(clippy::explicit_counter_loop)]
            for (metrics, estimator) in izip!(all_metrics, all_bindings) {
                let fold_i = i / all_hyperparams.len();
                let hyperparams_i = i % all_hyperparams.len();
                let hyperparams = &all_hyperparams[hyperparams_i];
                let metric = *metrics.get(&target_metric).unwrap();
                fit_times[hyperparams_i][fold_i] = *metrics.get("fit_time").unwrap();
                score_times[hyperparams_i][fold_i] = *metrics.get("score_time").unwrap();
                test_scores[hyperparams_i][fold_i] = metric;
                fold_scores[fold_i][hyperparams_i] = metric;

                if metric > best_metric {
                    best_index = hyperparams_i;
                    best_metric = metric;
                    best_metrics = Some(metrics);
                    best_hyperparams = Some(hyperparams);
                    best_estimator = Some(estimator);
                }
                i += 1;
            }

            search_results.insert("best_index".to_string(), json!(best_index));
            search_results.insert(
                "mean_fit_time".to_string(),
                json!(fit_times
                    .iter()
                    .map(|v| ArrayView1::from(v).mean().unwrap())
                    .collect::<Vec<f32>>()),
            );
            search_results.insert(
                "std_fit_time".to_string(),
                json!(fit_times
                    .iter()
                    .map(|v| ArrayView1::from(v).std(0.))
                    .collect::<Vec<f32>>()),
            );
            search_results.insert(
                "mean_score_time".to_string(),
                json!(score_times
                    .iter()
                    .map(|v| ArrayView1::from(v).mean().unwrap())
                    .collect::<Vec<f32>>()),
            );
            search_results.insert(
                "std_score_time".to_string(),
                json!(score_times
                    .iter()
                    .map(|v| ArrayView1::from(v).std(0.))
                    .collect::<Vec<f32>>()),
            );
            search_results.insert(
                "mean_test_score".to_string(),
                json!(test_scores
                    .iter()
                    .map(|v| ArrayView1::from(v).mean().unwrap())
                    .collect::<Vec<f32>>()),
            );
            search_results.insert(
                "std_test_score".to_string(),
                json!(test_scores
                    .iter()
                    .map(|v| ArrayView1::from(v).std(0.))
                    .collect::<Vec<f32>>()),
            );
            for (k, score) in fold_scores.iter().enumerate() {
                search_results.insert(format!("split{k}_test_score"), json!(score));
            }
            for param in best_hyperparams.unwrap().keys() {
                let params: Vec<serde_json::Value> = all_hyperparams
                    .iter()
                    .map(|hyperparams| json!(hyperparams.get(param).unwrap()))
                    .collect();
                search_results.insert(format!("param_{param}"), json!(params));
            }
            let mut metrics = IndexMap::new();
            for (key, value) in best_metrics.as_ref().unwrap() {
                metrics.insert(key.to_string(), json!(value));
            }
            metrics.insert("search_results".to_string(), json!(search_results));

            self.bindings = best_estimator;
            self.hyperparams = JsonB(json!(best_hyperparams.unwrap().clone()));
            self.metrics = Some(JsonB(json!(metrics)));
        };

        Spi::get_one_with_args::<i64>(
            "UPDATE pgml.models SET hyperparams = $1, metrics = $2 WHERE id = $3 RETURNING id",
            vec![
                (
                    PgBuiltInOids::JSONBOID.oid(),
                    JsonB(self.hyperparams.0.clone()).into_datum(),
                ),
                (
                    PgBuiltInOids::JSONBOID.oid(),
                    JsonB(self.metrics.as_ref().unwrap().0.clone()).into_datum(),
                ),
                (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
            ],
        )
        .unwrap();

        // Save the bindings.
        Spi::get_one_with_args::<i64>(
            "INSERT INTO pgml.files (model_id, path, part, data) VALUES($1, 'estimator.rmp', 0, $2) RETURNING id",
            vec![
                (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
                (
                    PgBuiltInOids::BYTEAOID.oid(),
                    self.bindings.as_ref().unwrap().to_bytes().into_datum(),
                ),
            ],
        )
        .unwrap();
    }

    pub fn numeric_encode_features(&self, rows: &[pgrx::datum::AnyElement]) -> Vec<f32> {
        // TODO handle FLOAT4[] as if it were pgrx::datum::AnyElement, skipping all this, and going straight to predict
        let mut features = Vec::new(); // TODO pre-allocate space
        for row in rows {
            match row.oid() {
                pgrx_pg_sys::RECORDOID => {
                    let tuple = unsafe { PgHeapTuple::from_composite_datum(row.datum()) };
                    for (i, column) in self.snapshot.features().enumerate() {
                        let index = NonZeroUsize::new(i + 1).unwrap();
                        let attribute = tuple.get_attribute_by_index(index).unwrap();
                        match &column.statistics.categories {
                            Some(_categories) => {
                                let key = match attribute.atttypid {
                                    pgrx_pg_sys::UNKNOWNOID => {
                                        error!("Type information missing for column: {:?}. If this is intended to be a TEXT or other categorical column, you will need to explicitly cast it, e.g. change `{:?}` to `CAST({:?} AS TEXT)`.", column.name, column.name, column.name);
                                    }
                                    pgrx_pg_sys::TEXTOID | pgrx_pg_sys::VARCHAROID | pgrx_pg_sys::BPCHAROID => {
                                        let element: Result<Option<String>, TryFromDatumError> =
                                            tuple.get_by_index(index);
                                        element.unwrap().unwrap_or(snapshot::NULL_CATEGORY_KEY.to_string())
                                    }
                                    pgrx_pg_sys::BOOLOID => {
                                        let element: Result<Option<bool>, TryFromDatumError> =
                                            tuple.get_by_index(index);
                                        element
                                            .unwrap()
                                            .map_or(snapshot::NULL_CATEGORY_KEY.to_string(), |k| k.to_string())
                                    }
                                    pgrx_pg_sys::INT2OID => {
                                        let element: Result<Option<i16>, TryFromDatumError> = tuple.get_by_index(index);
                                        element
                                            .unwrap()
                                            .map_or(snapshot::NULL_CATEGORY_KEY.to_string(), |k| k.to_string())
                                    }
                                    pgrx_pg_sys::INT4OID => {
                                        let element: Result<Option<i32>, TryFromDatumError> = tuple.get_by_index(index);
                                        element
                                            .unwrap()
                                            .map_or(snapshot::NULL_CATEGORY_KEY.to_string(), |k| k.to_string())
                                    }
                                    pgrx_pg_sys::INT8OID => {
                                        let element: Result<Option<i64>, TryFromDatumError> = tuple.get_by_index(index);
                                        element
                                            .unwrap()
                                            .map_or(snapshot::NULL_CATEGORY_KEY.to_string(), |k| k.to_string())
                                    }
                                    pgrx_pg_sys::FLOAT4OID => {
                                        let element: Result<Option<f32>, TryFromDatumError> = tuple.get_by_index(index);
                                        element
                                            .unwrap()
                                            .map_or(snapshot::NULL_CATEGORY_KEY.to_string(), |k| k.to_string())
                                    }
                                    pgrx_pg_sys::FLOAT8OID => {
                                        let element: Result<Option<f64>, TryFromDatumError> = tuple.get_by_index(index);
                                        element
                                            .unwrap()
                                            .map_or(snapshot::NULL_CATEGORY_KEY.to_string(), |k| k.to_string())
                                    }
                                    pgrx_pg_sys::NUMERICOID => {
                                        let element: Result<Option<AnyNumeric>, TryFromDatumError> =
                                            tuple.get_by_index(index);
                                        element
                                            .unwrap()
                                            .map_or(snapshot::NULL_CATEGORY_KEY.to_string(), |k| k.to_string())
                                    }
                                    _ => error!(
                                        "Unsupported type for categorical column: {:?}. oid: {:?}",
                                        column.name, attribute.atttypid
                                    ),
                                };
                                let value = column.get_category_value(&key);
                                features.push(value);
                            }
                            None => {
                                match attribute.atttypid {
                                    pgrx_pg_sys::UNKNOWNOID => {
                                        error!("Type information missing for column: {:?}. If this is intended to be a FLOAT4 or other numeric column, you will need to explicitly cast it, e.g. change `{:?}` to `CAST({:?} AS FLOAT4)`.", column.name, column.name, column.name);
                                    }
                                    pgrx_pg_sys::BOOLOID => {
                                        let element: Result<Option<bool>, TryFromDatumError> =
                                            tuple.get_by_index(index);
                                        features.push(element.unwrap().map_or(f32::NAN, |v| v as u8 as f32));
                                    }
                                    pgrx_pg_sys::INT2OID => {
                                        let element: Result<Option<i16>, TryFromDatumError> = tuple.get_by_index(index);
                                        features.push(element.unwrap().map_or(f32::NAN, |v| v as f32));
                                    }
                                    pgrx_pg_sys::INT4OID => {
                                        let element: Result<Option<i32>, TryFromDatumError> = tuple.get_by_index(index);
                                        features.push(element.unwrap().map_or(f32::NAN, |v| v as f32));
                                    }
                                    pgrx_pg_sys::INT8OID => {
                                        let element: Result<Option<i64>, TryFromDatumError> = tuple.get_by_index(index);
                                        features.push(element.unwrap().map_or(f32::NAN, |v| v as f32));
                                    }
                                    pgrx_pg_sys::FLOAT4OID => {
                                        let element: Result<Option<f32>, TryFromDatumError> = tuple.get_by_index(index);
                                        features.push(element.unwrap().map_or(f32::NAN, |v| v));
                                    }
                                    pgrx_pg_sys::FLOAT8OID => {
                                        let element: Result<Option<f64>, TryFromDatumError> = tuple.get_by_index(index);
                                        features.push(element.unwrap().map_or(f32::NAN, |v| v as f32));
                                    }
                                    pgrx_pg_sys::NUMERICOID => {
                                        let element: Result<Option<AnyNumeric>, TryFromDatumError> =
                                            tuple.get_by_index(index);
                                        features.push(element.unwrap().map_or(f32::NAN, |v| v.try_into().unwrap()));
                                    }
                                    // TODO handle NULL to NaN for arrays
                                    pgrx_pg_sys::BOOLARRAYOID => {
                                        let element: Result<Option<Vec<bool>>, TryFromDatumError> =
                                            tuple.get_by_index(index);
                                        for j in element.as_ref().unwrap().as_ref().unwrap() {
                                            features.push(*j as i8 as f32);
                                        }
                                    }
                                    pgrx_pg_sys::INT2ARRAYOID => {
                                        let element: Result<Option<Vec<i16>>, TryFromDatumError> =
                                            tuple.get_by_index(index);
                                        for j in element.as_ref().unwrap().as_ref().unwrap() {
                                            features.push(*j as f32);
                                        }
                                    }
                                    pgrx_pg_sys::INT4ARRAYOID => {
                                        let element: Result<Option<Vec<i32>>, TryFromDatumError> =
                                            tuple.get_by_index(index);
                                        for j in element.as_ref().unwrap().as_ref().unwrap() {
                                            features.push(*j as f32);
                                        }
                                    }
                                    pgrx_pg_sys::INT8ARRAYOID => {
                                        let element: Result<Option<Vec<i64>>, TryFromDatumError> =
                                            tuple.get_by_index(index);
                                        for j in element.as_ref().unwrap().as_ref().unwrap() {
                                            features.push(*j as f32);
                                        }
                                    }
                                    pgrx_pg_sys::FLOAT4ARRAYOID => {
                                        let element: Result<Option<Vec<f32>>, TryFromDatumError> =
                                            tuple.get_by_index(index);
                                        for j in element.as_ref().unwrap().as_ref().unwrap() {
                                            features.push(*j);
                                        }
                                    }
                                    pgrx_pg_sys::FLOAT8ARRAYOID => {
                                        let element: Result<Option<Vec<f64>>, TryFromDatumError> =
                                            tuple.get_by_index(index);
                                        for j in element.as_ref().unwrap().as_ref().unwrap() {
                                            features.push(*j as f32);
                                        }
                                    }
                                    pgrx_pg_sys::NUMERICARRAYOID => {
                                        let element: Result<Option<Vec<AnyNumeric>>, TryFromDatumError> =
                                            tuple.get_by_index(index);
                                        for j in element.as_ref().unwrap().as_ref().unwrap() {
                                            features.push(j.clone().try_into().unwrap());
                                        }
                                    }
                                    _ => error!(
                                        "Unsupported type for quantitative column: {:?}. oid: {:?}",
                                        column.name, attribute.atttypid
                                    ),
                                }
                            }
                        }
                    }
                }
                _ => error!("This preprocessing requires Postgres `record` types created with `row()`."),
            }
        }
        features
    }

    pub fn predict(&self, features: &[f32]) -> Result<f32> {
        Ok(self.predict_batch(features)?[0])
    }

    pub fn predict_proba(&self, features: &[f32]) -> Result<Vec<f32>> {
        match self.project.task {
            Task::regression => bail!("You can't predict probabilities for a regression model"),
            Task::classification => self
                .bindings
                .as_ref()
                .unwrap()
                .predict_proba(features, self.num_features),
            _ => bail!("no predict_proba for huggingface"),
        }
    }

    pub fn predict_joint(&self, features: &[f32]) -> Result<Vec<f32>> {
        match self.project.task {
            Task::regression => self
                .bindings
                .as_ref()
                .unwrap()
                .predict(features, self.num_features, self.num_classes),
            Task::classification => {
                bail!("You can't predict joint probabilities for a classification model")
            }
            _ => bail!("no predict_joint for huggingface"),
        }
    }

    pub fn predict_batch(&self, features: &[f32]) -> Result<Vec<f32>> {
        self.bindings
            .as_ref()
            .unwrap()
            .predict(features, self.num_features, self.num_classes)
    }

    pub fn decompose(&self, vector: &[f32]) -> Result<Vec<f32>> {
        self.bindings
            .as_ref()
            .unwrap()
            .predict(vector, self.num_features, self.num_classes)
    }
}
