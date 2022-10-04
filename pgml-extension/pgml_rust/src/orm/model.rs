use std::collections::HashMap;

use ::linfa::prelude::{BinaryClassification, Pr, SingleTargetRegression, ToConfusionMatrix};
use ndarray::ArrayView1;
use pgx::*;
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

    fn hyperparams(&self) -> &Hyperparams {
        let hyperparams: &serde_json::Value = &self.hyperparams.0;
        hyperparams.as_object().unwrap()
    }

    // TODO hyperparam search inside fit
    //use itertools::Itertools;

    // pub fn xgboost_search(
    //     task: Task,
    //     search: Search, // TODO: support random, only grid supported at the moment
    //     dataset: &Dataset,
    //     hyperparams: &Hyperparams,
    //     search_params: &Hyperparams,
    // ) -> (BoosterBox, Hyperparams) {
    //     let mut param_names = Vec::new();
    //     let mut search_field = Vec::new();

    //     // Iterate in order.
    //     for key in search_params.keys().into_iter() {
    //         param_names.push(key.to_string());

    //         let values: Vec<serde_json::Value> =
    //             search_params.get(key).unwrap().as_array().unwrap().to_vec();

    //         search_field.push(values);
    //     }

    //     // Grid search
    //     let search_field = search_field.into_iter().multi_cartesian_product();
    //     let mut best_metric = 0.0;
    //     let mut best_params = Hyperparams::new();
    //     let mut best_bst: Option<BoosterBox> = None;

    //     for combo in search_field {
    //         let mut candidates = Hyperparams::new();

    //         for (idx, value) in combo.iter().enumerate() {
    //             // Get the parameter name
    //             let k = param_names[idx].clone();
    //             candidates.insert(k, value.clone());
    //         }

    //         candidates.extend(hyperparams.clone());

    //         let bst = xgboost_train(task, dataset, &candidates);

    //         // Test
    //         let y_hat =
    //             Array1::from_shape_vec(dataset.num_test_rows, xgboost_test(&bst, dataset)).unwrap();
    //         let y_test =
    //             Array1::from_shape_vec(dataset.num_test_rows, dataset.y_test().to_vec()).unwrap();

    //         let metrics = calc_metrics(&y_test, &y_hat, dataset.distinct_labels(), task);

    //         // Compare
    //         let metric = match task {
    //             Task::regression => metrics.get("r2").unwrap(),

    //             Task::classification => metrics.get("f1").unwrap(),
    //         };

    //         if metric > &best_metric {
    //             best_metric = *metric;
    //             best_params = candidates.clone();
    //             best_bst = Some(bst);
    //         }
    //     }

    //     // Return the best
    //     (best_bst.unwrap(), best_params)
    // }

    fn fit(&mut self, project: &Project, dataset: &Dataset) {
        let fit = match self.runtime {
            Runtime::rust => {
                match project.task {
                    Task::regression => match self.algorithm {
                        Algorithm::xgboost => xgboost::fit_regression,
                        Algorithm::lightgbm => lightgbm::fit_regression,
                        Algorithm::linear => linfa::LinearRegression::fit,
                        _ => todo!(),
                    },
                    Task::classification => {
                        match self.algorithm {
                            Algorithm::xgboost => xgboost::fit_classification,
                            Algorithm::lightgbm => lightgbm::fit_classification,
                            // Algorithm::linear => linfa::LogisticRegression::fit,
                            _ => todo!(),
                        }
                    }
                }
            }
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
        };
        let estimator = fit(dataset, self.hyperparams());

        // info!("{:?}", 1 == 1.0);
        // Caculate model metrics used to evaluate its performance.
        let y_hat = estimator.predict_batch(dataset.x_test());
        let y_test = dataset.y_test();

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
                if dataset.distinct_labels() == 2 {
                    let y_hat = ArrayView1::from(&y_hat).mapv(Pr::new);
                    let y_test: Vec<bool> = y_test.iter().map(|&i| i == 1.).collect();
                    metrics.insert(
                        "roc_auc".to_string(),
                        y_hat.roc(&y_test).unwrap().area_under_curve(),
                    );
                    metrics.insert("log_loss".to_string(), y_hat.log_loss(&y_test).unwrap());
                }

                let y_hat: Vec<usize> = y_hat.into_iter().map(|i| i.round() as usize).collect();
                let y_test: Vec<usize> = y_test.iter().map(|i| i.round() as usize).collect();
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
        // let metrics = test(estimator, dataset, project.task);
        self.metrics = Some(JsonB(json!(metrics)));
        Spi::get_one_with_args::<i64>(
            "UPDATE pgml.models SET metrics = $1 WHERE id = $2 RETURNING id",
            vec![
                (
                    PgBuiltInOids::JSONBOID.oid(),
                    JsonB(json!(metrics)).into_datum(),
                ),
                (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
            ],
        )
        .unwrap();

        // Save the estimator.
        let bytes = estimator.to_bytes();
        Spi::get_one_with_args::<i64>(
            "INSERT INTO pgml.files (model_id, path, part, data) VALUES($1, 'estimator.rmp', 0, $2) RETURNING id",
            vec![
                (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
                (PgBuiltInOids::BYTEAOID.oid(), bytes.into_datum()),
            ]
            ).unwrap();
    }
}
