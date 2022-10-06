use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::time::Instant;

use ::linfa::prelude::{BinaryClassification, Pr, SingleTargetRegression, ToConfusionMatrix};
use itertools::{izip, Itertools};
use ndarray::ArrayView1;
use pgx::*;
use rand::prelude::SliceRandom;
use serde_json::json;

use crate::bindings::*;
use crate::orm::Dataset;
use crate::orm::*;

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
}

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "Model {{ id: {}, algorithm: {:?}, runtime: {:?} }}",
            self.id, self.algorithm, self.runtime
        )
    }
}

impl Model {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        project: &Project,
        snapshot: &Snapshot,
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
            None => match algorithm {
                Algorithm::xgboost => Runtime::rust,
                Algorithm::lightgbm => Runtime::rust,
                Algorithm::linear => match project.task {
                    Task::classification => Runtime::python,
                    Task::regression => Runtime::rust,
                },
                _ => Runtime::python,
            },
        };

        let dataset = snapshot.dataset();
        let status = Status::in_progress;
        // Create the model record.
        Spi::connect(|client| {
            let result = client.select("
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
                  (PgBuiltInOids::INT4OID.oid(), dataset.num_features.into_datum()),
              ])
          ).first();
            if !result.is_empty() {
                model = Some(Model {
                    id: result.get_datum(1).unwrap(),
                    project_id: result.get_datum(2).unwrap(),
                    snapshot_id: result.get_datum(3).unwrap(),
                    algorithm, // 4
                    runtime,   // 5
                    hyperparams: result.get_datum(6).unwrap(),
                    status, // 6,
                    metrics: result.get_datum(8),
                    search, // 9
                    search_params: result.get_datum(10).unwrap(),
                    search_args: result.get_datum(11).unwrap(),
                    created_at: result.get_datum(12).unwrap(),
                    updated_at: result.get_datum(13).unwrap(),
                });
            }

            Ok(Some(1))
        });

        let mut model = model.unwrap();
        info!("pgml.model: {}", model);

        model.fit(project, &dataset);

        Spi::connect(|client| {
            client.select(
                "UPDATE pgml.models SET status = $1::pgml.status WHERE id = $2",
                Some(1),
                Some(vec![
                    (
                        PgBuiltInOids::TEXTOID.oid(),
                        Status::successful.to_string().into_datum(),
                    ),
                    (PgBuiltInOids::INT8OID.oid(), model.id.into_datum()),
                ]),
            );

            Ok(Some(1))
        });

        model
    }

    fn get_fit_function(&self, project: &Project) -> crate::bindings::Fit {
        match self.runtime {
            Runtime::rust => match project.task {
                Task::regression => match self.algorithm {
                    Algorithm::xgboost => xgboost::fit_regression,
                    Algorithm::lightgbm => lightgbm::fit_regression,
                    Algorithm::linear => linfa::LinearRegression::fit,
                    _ => todo!(),
                },
                Task::classification => match self.algorithm {
                    Algorithm::xgboost => xgboost::fit_classification,
                    Algorithm::lightgbm => lightgbm::fit_classification,
                    Algorithm::linear => linfa::LogisticRegression::fit,
                    _ => todo!(),
                },
            },
            Runtime::python => match project.task {
                Task::regression => match self.algorithm {
                    Algorithm::linear => sklearn::linear_regression,
                    Algorithm::lasso => sklearn::lasso_regression,
                    Algorithm::svm => sklearn::svm_regression,
                    Algorithm::elastic_net => sklearn::elastic_net_regression,
                    Algorithm::ridge => sklearn::ridge_regression,
                    Algorithm::random_forest => sklearn::random_forest_regression,
                    Algorithm::xgboost => sklearn::xgboost_regression,
                    Algorithm::xgboost_random_forest => sklearn::xgboost_random_forest_regression,
                    Algorithm::orthogonal_matching_pursuit => {
                        sklearn::orthogonal_matching_persuit_regression
                    }
                    Algorithm::bayesian_ridge => sklearn::bayesian_ridge_regression,
                    Algorithm::automatic_relevance_determination => {
                        sklearn::automatic_relevance_determination_regression
                    }
                    Algorithm::stochastic_gradient_descent => {
                        sklearn::stochastic_gradient_descent_regression
                    }
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
                    Algorithm::gradient_boosting_trees => {
                        sklearn::gradient_boosting_trees_regression
                    }
                    Algorithm::hist_gradient_boosting => sklearn::hist_gradient_boosting_regression,
                    Algorithm::least_angle => sklearn::least_angle_regression,
                    Algorithm::lasso_least_angle => sklearn::lasso_least_angle_regression,
                    Algorithm::linear_svm => sklearn::linear_svm_regression,
                    Algorithm::lightgbm => sklearn::lightgbm_regression,
                    _ => panic!("{:?} does not support regression", self.algorithm),
                },
                Task::classification => match self.algorithm {
                    Algorithm::linear => sklearn::linear_classification,
                    Algorithm::svm => sklearn::svm_classification,
                    Algorithm::ridge => sklearn::ridge_classification,
                    Algorithm::random_forest => sklearn::random_forest_classification,
                    Algorithm::xgboost => sklearn::xgboost_classification,
                    Algorithm::xgboost_random_forest => {
                        sklearn::xgboost_random_forest_classification
                    }
                    Algorithm::stochastic_gradient_descent => {
                        sklearn::stochastic_gradient_descent_classification
                    }
                    Algorithm::perceptron => sklearn::perceptron_classification,
                    Algorithm::passive_aggressive => sklearn::passive_aggressive_classification,
                    Algorithm::gaussian_process => sklearn::gaussian_process,
                    Algorithm::nu_svm => sklearn::nu_svm_classification,
                    Algorithm::ada_boost => sklearn::ada_boost_classification,
                    Algorithm::bagging => sklearn::bagging_classification,
                    Algorithm::extra_trees => sklearn::extra_trees_classification,
                    Algorithm::gradient_boosting_trees => {
                        sklearn::gradient_boosting_trees_classification
                    }
                    Algorithm::hist_gradient_boosting => {
                        sklearn::hist_gradient_boosting_classification
                    }
                    Algorithm::linear_svm => sklearn::linear_svm_classification,
                    Algorithm::lightgbm => sklearn::lightgbm_classification,
                    _ => panic!("{:?} does not support classification", self.algorithm),
                },
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
                error!("`{key}` cannot be present in both hyperparams and search_params. Please choose one or the other.");
            }
            all_hyperparam_names.push(key.to_string());
            all_hyperparam_values.push(values.as_array().unwrap().to_vec());
        }

        // The search space is all possible combinations
        let all_hyperparam_values: Vec<Vec<serde_json::Value>> = all_hyperparam_values
            .into_iter()
            .multi_cartesian_product()
            .collect();
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

    #[allow(clippy::format_push_string)]
    fn fit(&mut self, project: &Project, dataset: &Dataset) {
        let fit = self.get_fit_function(project);

        let mut n_iter: usize = 10;
        let mut cv: usize = 1;
        for (key, value) in self.search_args.0.as_object().unwrap() {
            match key.as_str() {
                "n_iter" => n_iter = value.as_i64().unwrap().try_into().unwrap(),
                "cv" => cv = value.as_i64().unwrap().try_into().unwrap(),
                _ => error!("Unknown search_args => {:?}: {:?}", key, value),
            }
        }

        // info!(
        //     "pgml.hyperparam searches: {}, cross validation folds: {}",
        //     all_hyperparam_values.len(),
        //     1
        // );

        let all_hyperparams = self.get_all_hyperparams(n_iter);
        let mut all_estimators = Vec::with_capacity(all_hyperparams.len());
        let mut all_metrics = Vec::with_capacity(all_hyperparams.len());
        // let mut fit_times = Vec::with_capacity(all_hyperparams.len());
        // let mut test_times = Vec::with_capacity(all_hyperparams.len());
        // let search_results = HashMap::new();
        // search_results.set("params", all_hyperparams);
        // search_results.set("n_splits", cv);
        // search_results.set("mean_fit_time", Vec::with_capacity(all_hyperparams.len()));
        // search_results.set("std_fit_time", Vec::with_capacity(all_hyperparams.len()));
        // search_results.set("mean_score_time", Vec::with_capacity(all_hyperparams.len()));
        // search_results.set("std_score_time", Vec::with_capacity(all_hyperparams.len()));
        // search_results.set("mean_test_score", Vec::with_capacity(all_hyperparams.len()));
        // search_results.set("std_test_score", Vec::with_capacity(all_hyperparams.len()));
        // search_results.set("rank_test_score", Vec::with_capacity(all_hyperparams.len()));
        // for param in all_hyperparams.first().keys() {
        //     let key = format!("param_{}", param);
        //     search_results.set(key, Vec::with_capacity(all_hyperparams.len()))
        // }
        // for k in 0..cv {
        //     search_results.set(format!("split{k}_test_score"), Vec::with_capacity(all_hyperparams.len()))
        // }

        for k in 0..cv {
            // TODO, don't fold when CV < 2
            let fold = dataset.fold(k, cv);

            for hyperparams in &all_hyperparams {
                let now = Instant::now();

                // Fit the estimator on the data with these hyperparams
                let estimator = fit(&fold, hyperparams);

                // Test the estimator on the data
                let y_hat = estimator.predict_batch(&fold.x_test);
                let y_test = &fold.y_test;

                // Caculate metrics to evaluate this estimator and its hyperparams
                let mut metrics = HashMap::new();
                match project.task {
                    Task::regression => {
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
                        if fold.num_distinct_labels == 2 {
                            let y_hat = ArrayView1::from(&y_hat).mapv(Pr::new);
                            let y_test: Vec<bool> = y_test.iter().map(|&i| i == 1.).collect();
                            metrics.insert(
                                "roc_auc".to_string(),
                                y_hat.roc(&y_test).unwrap().area_under_curve(),
                            );
                            metrics
                                .insert("log_loss".to_string(), y_hat.log_loss(&y_test).unwrap());
                        }

                        let y_hat: Vec<usize> =
                            y_hat.into_iter().map(|i| i.round() as usize).collect();
                        let y_test: Vec<usize> =
                            y_test.iter().map(|i| i.round() as usize).collect();
                        let y_hat = ArrayView1::from(&y_hat);
                        let y_test = ArrayView1::from(&y_test);
                        let confusion_matrix = y_hat.confusion_matrix(y_test).unwrap();
                        metrics.insert("f1".to_string(), confusion_matrix.f1_score());
                        metrics.insert("precision".to_string(), confusion_matrix.precision());
                        metrics.insert("recall".to_string(), confusion_matrix.recall());
                        metrics.insert("accuracy".to_string(), confusion_matrix.accuracy());
                        metrics.insert("mcc".to_string(), confusion_matrix.mcc());
                    }
                }
                let mut buffer = "pgml.hyperparams: {".to_string();
                buffer.push_str(
                    &hyperparams
                        .iter()
                        .sorted_by_key(|x| x.0)
                        .map(|(key, value)| format!("{:?}: {}", key, value))
                        .join(", "),
                );
                buffer.push_str("}\n  pgml.metrics: {");
                buffer.push_str(
                    &metrics
                        .iter()
                        .sorted_by_key(|x| x.0)
                        .map(|(key, value)| format!("{:?}: {:?}", key, value))
                        .join(", "),
                );
                buffer.push_str(&format!("}}\n  pgml.fitting time: {:?}", now.elapsed()));
                info!("{}", buffer);

                all_metrics.push(metrics);
                all_estimators.push(estimator);
            }
        }
        // Find the best estimator, hyperparams and metrics
        // TODO expose this as a parameter on the project
        let target_metric = match project.task {
            Task::regression => "r2",
            Task::classification => "f1",
        };
        let mut best_metric = 0.0;
        // let mut best_index = 0;
        let mut best_metrics = None;
        let mut best_hyperparams = None;
        let mut best_estimator = None;
        // let mut i = 0;
        for (metrics, hyperparams, estimator) in izip!(all_metrics, all_hyperparams, all_estimators)
        {
            let metric = *metrics.get(target_metric).unwrap();
            if metric > best_metric {
                // best_index = i;
                best_metric = metric;
                best_metrics = Some(metrics);
                best_hyperparams = Some(hyperparams);
                best_estimator = Some(estimator);
            }
            // i += 1;
        }

        // search_results.set("best_index", best_index);

        let best_metrics = best_metrics.unwrap();
        let best_hyperparams = best_hyperparams.unwrap();
        self.metrics = Some(JsonB(json!(best_metrics)));
        self.hyperparams = JsonB(json!(best_hyperparams));
        Spi::get_one_with_args::<i64>(
            "UPDATE pgml.models SET hyperparams = $1, metrics = $2 WHERE id = $3 RETURNING id",
            vec![
                (
                    PgBuiltInOids::JSONBOID.oid(),
                    JsonB(json!(&best_hyperparams)).into_datum(),
                ),
                (
                    PgBuiltInOids::JSONBOID.oid(),
                    JsonB(json!(&best_metrics)).into_datum(),
                ),
                (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
            ],
        )
        .unwrap();

        // Save the estimator.
        let bytes = best_estimator.unwrap().to_bytes();
        Spi::get_one_with_args::<i64>(
            "INSERT INTO pgml.files (model_id, path, part, data) VALUES($1, 'estimator.rmp', 0, $2) RETURNING id",
            vec![
                (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
                (PgBuiltInOids::BYTEAOID.oid(), bytes.into_datum()),
            ]
            ).unwrap();
    }
}
