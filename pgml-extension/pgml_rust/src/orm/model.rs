use std::str::FromStr;

use ndarray::{Array1, Array2};
use pgx::*;
use serde_json::json;
use xgboost::{parameters, Booster, DMatrix};

use crate::orm::estimator::BoosterBox;
use crate::orm::Algorithm;
use crate::orm::Dataset;
use crate::orm::Estimator;
use crate::orm::Project;
use crate::orm::Search;
use crate::orm::Snapshot;
use crate::orm::Task;

/// Get a floating point hyperparameter.
macro_rules! hyperparam_f32 {
    ($name:tt, $hyperparams:tt, $default:tt) => {
        let $name = match $hyperparams.get("$name") {
            Some($name) => $name.as_f64().unwrap_or($default) as f32,
            None => $default,
        };
    };
}

/// Get a usize (u64) hyperparameter.
macro_rules! hyperparam_usize {
    ($name:tt, $hyperparams:tt, $default:tt) => {
        let $name = match $hyperparams.get("$name") {
            Some($name) => $name.as_u64().unwrap_or($default) as usize,
            None => $default,
        };
    };
}

/// Get a boolean hyperparameter.
macro_rules! hyperparam_bool {
    ($name:tt, $hyperparams:tt, $default:tt) => {
        let $name = match $hyperparams.get("$name") {
            Some($name) => $name.as_bool().unwrap_or($default),
            None => $default,
        };
    };
}

/// Split the dataset into test and train parts.
macro_rules! train_test_split {
    ($dataset:tt, $x_train:tt, $y_train:tt) => {
        let $x_train = Array2::from_shape_vec(
            ($dataset.num_train_rows, $dataset.num_features),
            $dataset.x_train().to_vec(),
        )
        .unwrap();

        let $y_train =
            Array1::from_shape_vec($dataset.num_train_rows, $dataset.y_train().to_vec()).unwrap();
    };
}

/// Save the trained estimator in the DB.
macro_rules! save_estimator {
    ($estimator:tt, $self:tt) => {
        let bytes: Vec<u8> = rmp_serde::to_vec($estimator.as_ref().unwrap()).unwrap();
        Spi::get_one_with_args::<i64>(
          "INSERT INTO pgml_rust.files (model_id, path, part, data) VALUES($1, 'estimator.rmp', 0, $2) RETURNING id",
          vec![
              (PgBuiltInOids::INT8OID.oid(), $self.id.into_datum()),
              (PgBuiltInOids::BYTEAOID.oid(), bytes.into_datum()),
          ]
        ).unwrap();
    }
}

#[derive(Debug)]
pub struct Model {
    pub id: i64,
    pub project_id: i64,
    pub snapshot_id: i64,
    pub algorithm: Algorithm,
    pub hyperparams: JsonB,
    pub status: String,
    pub metrics: Option<JsonB>,
    pub search: Option<Search>,
    pub search_params: JsonB,
    pub search_args: JsonB,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    estimator: Option<Box<dyn Estimator>>,
}

impl Model {
    pub fn create(
        project: &Project,
        snapshot: &Snapshot,
        algorithm: Algorithm,
        hyperparams: JsonB,
        search: Option<Search>,
        search_params: JsonB,
        search_args: JsonB,
    ) -> Model {
        let mut model: Option<Model> = None;

        Spi::connect(|client| {
            let result = client.select("
          INSERT INTO pgml_rust.models (project_id, snapshot_id, algorithm, hyperparams, status, search, search_params, search_args) 
          VALUES ($1, $2, $3, $4, $5, $6::pgml_rust.search, $7, $8) 
          RETURNING id, project_id, snapshot_id, algorithm, hyperparams, status, metrics, search, search_params, search_args, created_at, updated_at;",
              Some(1),
              Some(vec![
                  (PgBuiltInOids::INT8OID.oid(), project.id.into_datum()),
                  (PgBuiltInOids::INT8OID.oid(), snapshot.id.into_datum()),
                  (PgBuiltInOids::TEXTOID.oid(), algorithm.to_string().into_datum()),
                  (PgBuiltInOids::JSONBOID.oid(), hyperparams.into_datum()),
                  (PgBuiltInOids::TEXTOID.oid(), "new".to_string().into_datum()),
                  (PgBuiltInOids::TEXTOID.oid(), search.into_datum()),
                  (PgBuiltInOids::JSONBOID.oid(), search_params.into_datum()),
                  (PgBuiltInOids::JSONBOID.oid(), search_args.into_datum()),
              ])
          ).first();
            if !result.is_empty() {
                model = Some(Model {
                    id: result.get_datum(1).unwrap(),
                    project_id: result.get_datum(2).unwrap(),
                    snapshot_id: result.get_datum(3).unwrap(),
                    algorithm: Algorithm::from_str(result.get_datum(4).unwrap()).unwrap(),
                    hyperparams: result.get_datum(5).unwrap(),
                    status: result.get_datum(6).unwrap(),
                    metrics: result.get_datum(7),
                    search, // TODO
                    search_params: result.get_datum(9).unwrap(),
                    search_args: result.get_datum(10).unwrap(),
                    created_at: result.get_datum(11).unwrap(),
                    updated_at: result.get_datum(12).unwrap(),
                    estimator: None,
                });
            }

            Ok(Some(1))
        });
        let mut model = model.unwrap();
        let dataset = snapshot.dataset();
        model.fit(project, &dataset);
        model.test(project, &dataset);
        model
    }

    #[allow(non_snake_case)]
    fn fit(&mut self, project: &Project, dataset: &Dataset) {
        let hyperparams: &serde_json::Value = &self.hyperparams.0;
        let hyperparams = hyperparams.as_object().unwrap();

        self.estimator = match self.algorithm {
            Algorithm::svm => {
                train_test_split!(dataset, x_train, y_train);
                hyperparam_f32!(eps, hyperparams, 0.1);
                hyperparam_f32!(C, hyperparams, 1.0);
                hyperparam_f32!(tol, hyperparams, 1e-3);
                hyperparam_usize!(epoch, hyperparams, 2);
                hyperparam_f32!(degree, hyperparams, 3.0);
                hyperparam_f32!(coef0, hyperparams, 0.0);

                let gamma = match hyperparams.get("gamma") {
                    Some(gamma) => match gamma.as_f64() {
                        Some(gamma) => gamma as f32,
                        None => {
                            match gamma.as_str().unwrap_or("scale") {
                                "scale" => 1.0 / dataset.num_features as f32 * x_train.var(0.0), // variance of population
                                "auto" => 1.0 / dataset.num_features as f32,
                                _ => 1.0 / dataset.num_features as f32 * x_train.var(0.0),
                            }
                        }
                    },

                    None => 1.0 / dataset.num_features as f32 * x_train.var(0.0),
                };

                let estimator: Option<Box<dyn Estimator>> = match project.task {
                    Task::regression => match hyperparams.get("kernel") {
                        Some(kernel) => match kernel.as_str().unwrap_or("linear") {
                            "poly" => Some(Box::new(
                                smartcore::svm::svr::SVR::fit(
                                    &x_train,
                                    &y_train,
                                    smartcore::svm::svr::SVRParameters::default()
                                        .with_eps(eps)
                                        .with_c(C)
                                        .with_tol(tol)
                                        .with_kernel(smartcore::svm::Kernels::polynomial(
                                            degree, gamma, coef0,
                                        )),
                                )
                                .unwrap(),
                            )),

                            "sigmoid" => Some(Box::new(
                                smartcore::svm::svr::SVR::fit(
                                    &x_train,
                                    &y_train,
                                    smartcore::svm::svr::SVRParameters::default()
                                        .with_eps(eps)
                                        .with_c(C)
                                        .with_tol(tol)
                                        .with_kernel(smartcore::svm::Kernels::sigmoid(
                                            gamma, coef0,
                                        )),
                                )
                                .unwrap(),
                            )),

                            "rbf" => Some(Box::new(
                                smartcore::svm::svr::SVR::fit(
                                    &x_train,
                                    &y_train,
                                    smartcore::svm::svr::SVRParameters::default()
                                        .with_eps(eps)
                                        .with_c(C)
                                        .with_tol(tol)
                                        .with_kernel(smartcore::svm::Kernels::rbf(gamma)),
                                )
                                .unwrap(),
                            )),

                            _ => Some(Box::new(
                                smartcore::svm::svr::SVR::fit(
                                    &x_train,
                                    &y_train,
                                    smartcore::svm::svr::SVRParameters::default()
                                        .with_eps(eps)
                                        .with_c(C)
                                        .with_tol(tol)
                                        .with_kernel(smartcore::svm::Kernels::linear()),
                                )
                                .unwrap(),
                            )),
                        },

                        None => Some(Box::new(
                            smartcore::svm::svr::SVR::fit(
                                &x_train,
                                &y_train,
                                smartcore::svm::svr::SVRParameters::default()
                                    .with_eps(eps)
                                    .with_c(C)
                                    .with_tol(tol)
                                    .with_kernel(smartcore::svm::Kernels::linear()),
                            )
                            .unwrap(),
                        )),
                    },

                    Task::classification => match hyperparams.get("kernel") {
                        Some(kernel) => match kernel.as_str().unwrap_or("linear") {
                            "poly" => Some(Box::new(
                                smartcore::svm::svc::SVC::fit(
                                    &x_train,
                                    &y_train,
                                    smartcore::svm::svc::SVCParameters::default()
                                        .with_epoch(epoch)
                                        .with_c(C)
                                        .with_tol(tol)
                                        .with_kernel(smartcore::svm::Kernels::polynomial(
                                            degree, gamma, coef0,
                                        )),
                                )
                                .unwrap(),
                            )),

                            "sigmoid" => Some(Box::new(
                                smartcore::svm::svc::SVC::fit(
                                    &x_train,
                                    &y_train,
                                    smartcore::svm::svc::SVCParameters::default()
                                        .with_epoch(epoch)
                                        .with_c(C)
                                        .with_tol(tol)
                                        .with_kernel(smartcore::svm::Kernels::sigmoid(
                                            gamma, coef0,
                                        )),
                                )
                                .unwrap(),
                            )),

                            "rbf" => Some(Box::new(
                                smartcore::svm::svc::SVC::fit(
                                    &x_train,
                                    &y_train,
                                    smartcore::svm::svc::SVCParameters::default()
                                        .with_epoch(epoch)
                                        .with_c(C)
                                        .with_tol(tol)
                                        .with_kernel(smartcore::svm::Kernels::rbf(gamma)),
                                )
                                .unwrap(),
                            )),

                            _ => Some(Box::new(
                                smartcore::svm::svc::SVC::fit(
                                    &x_train,
                                    &y_train,
                                    smartcore::svm::svc::SVCParameters::default()
                                        .with_epoch(epoch)
                                        .with_tol(tol)
                                        .with_kernel(smartcore::svm::Kernels::linear()),
                                )
                                .unwrap(),
                            )),
                        },

                        None => Some(Box::new(
                            smartcore::svm::svc::SVC::fit(
                                &x_train,
                                &y_train,
                                smartcore::svm::svc::SVCParameters::default()
                                    .with_epoch(epoch)
                                    .with_c(C)
                                    .with_tol(tol)
                                    .with_kernel(smartcore::svm::Kernels::linear()),
                            )
                            .unwrap(),
                        )),
                    },
                };

                save_estimator!(estimator, self);

                estimator
            }

            Algorithm::linear => {
                train_test_split!(dataset, x_train, y_train);

                let estimator: Option<Box<dyn Estimator>> = match project.task {
                    Task::regression => {
                        let params = smartcore::linear::linear_regression::LinearRegressionParameters::default()
                            .with_solver(match hyperparams.get("solver"){
                                Some(value) => match value.as_str().unwrap_or("qr") {
                                    "qr" => smartcore::linear::linear_regression::LinearRegressionSolverName::QR,
                                    "svd" => smartcore::linear::linear_regression::LinearRegressionSolverName::SVD,
                                    _ => smartcore::linear::linear_regression::LinearRegressionSolverName::QR,
                                }
                                None => smartcore::linear::linear_regression::LinearRegressionSolverName::QR,
                            });

                        Some(Box::new(
                            smartcore::linear::linear_regression::LinearRegression::fit(
                                &x_train, &y_train, params,
                            )
                            .unwrap(),
                        ))
                    }
                    Task::classification => {
                        hyperparam_f32!(alpha, hyperparams, 0.0);

                        let params = smartcore::linear::logistic_regression::LogisticRegressionParameters::default()
                            .with_alpha(alpha);

                        Some(Box::new(
                            smartcore::linear::logistic_regression::LogisticRegression::fit(
                                &x_train, &y_train, params,
                            )
                            .unwrap(),
                        ))
                    }
                };

                save_estimator!(estimator, self);

                estimator
            }

            Algorithm::xgboost => {
                let mut dtrain =
                    DMatrix::from_dense(dataset.x_train(), dataset.num_train_rows).unwrap();
                let mut dtest =
                    DMatrix::from_dense(dataset.x_test(), dataset.num_test_rows).unwrap();
                dtrain.set_labels(dataset.y_train()).unwrap();
                dtest.set_labels(dataset.y_test()).unwrap();

                // specify datasets to evaluate against during training
                let evaluation_sets = &[(&dtrain, "train"), (&dtest, "test")];

                // configure objectives, metrics, etc.
                let learning_params =
                    parameters::learning::LearningTaskParametersBuilder::default()
                        .objective(match project.task {
                            Task::regression => xgboost::parameters::learning::Objective::RegLinear,
                            Task::classification => {
                                xgboost::parameters::learning::Objective::MultiSoftmax(
                                    dataset.distinct_labels(),
                                )
                            }
                        })
                        .build()
                        .unwrap();

                // configure the tree-based learning model's parameters
                let tree_params = parameters::tree::TreeBoosterParametersBuilder::default()
                    .max_depth(match hyperparams.get("max_depth") {
                        Some(value) => value.as_u64().unwrap_or(2) as u32,
                        None => 2,
                    })
                    .eta(match hyperparams.get("eta") {
                        Some(value) => value.as_f64().unwrap_or(0.3) as f32,
                        None => match hyperparams.get("learning_rate") {
                            Some(value) => value.as_f64().unwrap_or(0.3) as f32,
                            None => 0.3,
                        },
                    })
                    .gamma(match hyperparams.get("gamma") {
                        Some(value) => value.as_f64().unwrap_or(0.0) as f32,
                        None => match hyperparams.get("min_split_loss") {
                            Some(value) => value.as_f64().unwrap_or(0.0) as f32,
                            None => 0.0,
                        },
                    })
                    .min_child_weight(match hyperparams.get("min_child_weight") {
                        Some(value) => value.as_f64().unwrap_or(1.0) as f32,
                        None => 1.0,
                    })
                    .max_delta_step(match hyperparams.get("max_delta_step") {
                        Some(value) => value.as_f64().unwrap_or(0.0) as f32,
                        None => 0.0,
                    })
                    .subsample(match hyperparams.get("subsample") {
                        Some(value) => value.as_f64().unwrap_or(1.0) as f32,
                        None => 1.0,
                    })
                    .lambda(match hyperparams.get("lambda") {
                        Some(value) => value.as_f64().unwrap_or(1.0) as f32,
                        None => 1.0,
                    })
                    .alpha(match hyperparams.get("alpha") {
                        Some(value) => value.as_f64().unwrap_or(0.0) as f32,
                        None => 0.0,
                    })
                    .tree_method(match hyperparams.get("tree_method") {
                        Some(value) => match value.as_str().unwrap_or("auto") {
                            "auto" => parameters::tree::TreeMethod::Auto,
                            "exact" => parameters::tree::TreeMethod::Exact,
                            "approx" => parameters::tree::TreeMethod::Approx,
                            "hist" => parameters::tree::TreeMethod::Hist,
                            _ => parameters::tree::TreeMethod::Auto,
                        },

                        None => parameters::tree::TreeMethod::Auto,
                    })
                    .sketch_eps(match hyperparams.get("sketch_eps") {
                        Some(value) => value.as_f64().unwrap_or(0.03) as f32,
                        None => 0.03,
                    })
                    .max_leaves(match hyperparams.get("max_leaves") {
                        Some(value) => value.as_u64().unwrap_or(0) as u32,
                        None => 0,
                    })
                    .max_bin(match hyperparams.get("max_bin") {
                        Some(value) => value.as_u64().unwrap_or(256) as u32,
                        None => 256,
                    })
                    .num_parallel_tree(match hyperparams.get("num_parallel_tree") {
                        Some(value) => value.as_u64().unwrap_or(1) as u32,
                        None => 1,
                    })
                    .grow_policy(match hyperparams.get("grow_policy") {
                        Some(value) => match value.as_str().unwrap_or("depthwise") {
                            "depthwise" => parameters::tree::GrowPolicy::Depthwise,
                            "lossguide" => parameters::tree::GrowPolicy::LossGuide,
                            _ => parameters::tree::GrowPolicy::Depthwise,
                        },

                        None => parameters::tree::GrowPolicy::Depthwise,
                    })
                    .build()
                    .unwrap();

                let linear_params = parameters::linear::LinearBoosterParametersBuilder::default()
                    .alpha(match hyperparams.get("alpha") {
                        Some(value) => value.as_f64().unwrap_or(0.0) as f32,
                        None => 0.0,
                    })
                    .lambda(match hyperparams.get("lambda") {
                        Some(value) => value.as_f64().unwrap_or(0.0) as f32,
                        None => 0.0,
                    })
                    .build()
                    .unwrap();

                let dart_params = parameters::dart::DartBoosterParametersBuilder::default()
                    .rate_drop(match hyperparams.get("rate_drop") {
                        Some(value) => value.as_f64().unwrap_or(0.0) as f32,
                        None => 0.0,
                    })
                    .one_drop(match hyperparams.get("one_drop") {
                        Some(value) => value.as_u64().unwrap_or(0) != 0,
                        None => false,
                    })
                    .skip_drop(match hyperparams.get("skip_drop") {
                        Some(value) => value.as_f64().unwrap_or(0.0) as f32,
                        None => 0.0,
                    })
                    .sample_type(match hyperparams.get("sample_type") {
                        Some(value) => match value.as_str().unwrap_or("uniform") {
                            "uniform" => parameters::dart::SampleType::Uniform,
                            "weighted" => parameters::dart::SampleType::Weighted,
                            _ => parameters::dart::SampleType::Uniform,
                        },
                        None => parameters::dart::SampleType::Uniform,
                    })
                    .normalize_type(match hyperparams.get("normalize_type") {
                        Some(value) => match value.as_str().unwrap_or("tree") {
                            "tree" => parameters::dart::NormalizeType::Tree,
                            "forest" => parameters::dart::NormalizeType::Forest,
                            _ => parameters::dart::NormalizeType::Tree,
                        },
                        None => parameters::dart::NormalizeType::Tree,
                    })
                    .build()
                    .unwrap();

                // overall configuration for Booster
                let booster_params = parameters::BoosterParametersBuilder::default()
                    .booster_type(match hyperparams.get("booster") {
                        Some(value) => match value.as_str().unwrap_or("gbtree") {
                            "gbtree" => parameters::BoosterType::Tree(tree_params),
                            "linear" => parameters::BoosterType::Linear(linear_params),
                            "dart" => parameters::BoosterType::Dart(dart_params),
                            _ => parameters::BoosterType::Tree(tree_params),
                        },
                        None => parameters::BoosterType::Tree(tree_params),
                    })
                    .learning_params(learning_params)
                    .verbose(true)
                    .build()
                    .unwrap();

                // overall configuration for training/evaluation
                let params = parameters::TrainingParametersBuilder::default()
                    .dtrain(&dtrain) // dataset to train with
                    .boost_rounds(match hyperparams.get("n_estimators") {
                        Some(value) => value.as_u64().unwrap_or(2) as u32,
                        None => 2,
                    }) // number of training iterations
                    .booster_params(booster_params) // model parameters
                    .evaluation_sets(Some(evaluation_sets)) // optional datasets to evaluate against in each iteration
                    .build()
                    .unwrap();

                // train model, and print evaluation data
                let bst = match Booster::train(&params) {
                    Ok(bst) => bst,
                    Err(err) => error!("{}", err),
                };

                let r: u64 = rand::random();
                let path = format!("/tmp/pgml_rust_{}.bin", r);

                bst.save(std::path::Path::new(&path)).unwrap();

                let bytes = std::fs::read(&path).unwrap();
                Spi::get_one_with_args::<i64>(
                  "INSERT INTO pgml_rust.files (model_id, path, part, data) VALUES($1, 'estimator.rmp', 0, $2) RETURNING id",
                  vec![
                      (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
                      (PgBuiltInOids::BYTEAOID.oid(), bytes.into_datum()),
                  ]
                ).unwrap();
                Some(Box::new(BoosterBox::new(bst)))
            }

            Algorithm::lasso => {
                train_test_split!(dataset, x_train, y_train);

                hyperparam_f32!(alpha, hyperparams, 1.0);
                hyperparam_bool!(normalize, hyperparams, false);
                hyperparam_f32!(tol, hyperparams, 1e-4);
                hyperparam_usize!(max_iter, hyperparams, 1000);

                let estimator: Option<Box<dyn Estimator>> = match project.task {
                    Task::regression => Some(Box::new(
                        smartcore::linear::lasso::Lasso::fit(
                            &x_train,
                            &y_train,
                            smartcore::linear::lasso::LassoParameters::default()
                                .with_alpha(alpha)
                                .with_normalize(normalize)
                                .with_tol(tol)
                                .with_max_iter(max_iter),
                        )
                        .unwrap(),
                    )),

                    Task::classification => panic!("Lasso only supports regression"),
                };

                save_estimator!(estimator, self);

                estimator
            }

            Algorithm::elastic_net => {
                train_test_split!(dataset, x_train, y_train);

                hyperparam_f32!(alpha, hyperparams, 1.0);
                hyperparam_f32!(l1_ratio, hyperparams, 0.5);
                hyperparam_bool!(normalize, hyperparams, false);
                hyperparam_f32!(tol, hyperparams, 1e-4);
                hyperparam_usize!(max_iter, hyperparams, 1000);

                let estimator: Option<Box<dyn Estimator>> = match project.task {
                    Task::regression => Some(Box::new(
                        smartcore::linear::elastic_net::ElasticNet::fit(
                            &x_train,
                            &y_train,
                            smartcore::linear::elastic_net::ElasticNetParameters::default()
                                .with_alpha(alpha)
                                .with_l1_ratio(l1_ratio)
                                .with_normalize(normalize)
                                .with_tol(tol)
                                .with_max_iter(max_iter),
                        )
                        .unwrap(),
                    )),

                    Task::classification => panic!("Elastic Net does not support classification"),
                };

                save_estimator!(estimator, self);

                estimator
            }

            Algorithm::ridge => {
                train_test_split!(dataset, x_train, y_train);
                hyperparam_f32!(alpha, hyperparams, 1.0);
                hyperparam_bool!(normalize, hyperparams, false);

                let solver = match hyperparams.get("solver") {
                    Some(solver) => match solver.as_str().unwrap_or("cholesky") {
                        "svd" => {
                            smartcore::linear::ridge_regression::RidgeRegressionSolverName::SVD
                        }
                        _ => {
                            smartcore::linear::ridge_regression::RidgeRegressionSolverName::Cholesky
                        }
                    },
                    None => smartcore::linear::ridge_regression::RidgeRegressionSolverName::SVD,
                };

                let estimator: Option<Box<dyn Estimator>> = match project.task {
                    Task::regression => Some(
                        Box::new(
                            smartcore::linear::ridge_regression::RidgeRegression::fit(
                                &x_train,
                                &y_train,
                                smartcore::linear::ridge_regression::RidgeRegressionParameters::default()
                                .with_alpha(alpha)
                                .with_normalize(normalize)
                                .with_solver(solver)
                            ).unwrap()
                        )
                    ),

                    Task::classification => panic!("Ridge does not support classification"),
                };

                save_estimator!(estimator, self);

                estimator
            }

            Algorithm::kmeans => {
                todo!();
                // train_test_split!(dataset, x_train, y_train);

                // let estimator: Option<Box<dyn Estimator>> = match project.task = {
                //     Task::regression => (

                //     ),

                //     Task::classification => (

                //     ),
                // }

                // save_estimator!(estimator, self);

                // estimator
            }

            Algorithm::dbscan => {
                todo!();

                // train_test_split!(dataset, x_train, y_train);

                // let estimator = Option<Box<dyn Estimator>> = match project.task {
                //     Task::regression => (

                //     ),

                //     Task::classification => (

                //     ),
                // }

                // save_estimator!(estimator, self);

                // estimator
            }

            Algorithm::knn => {
                train_test_split!(dataset, x_train, y_train);
                let algorithm = match hyperparams
                    .get("algorithm")
                    .unwrap_or(&serde_json::Value::from("linear_search"))
                    .as_str()
                    .unwrap_or("linear_search")
                {
                    "cover_tree" => smartcore::neighbors::KNNAlgorithmName::CoverTree,
                    _ => smartcore::neighbors::KNNAlgorithmName::LinearSearch,
                };
                let weight = match hyperparams
                    .get("weight")
                    .unwrap_or(&serde_json::Value::from("uniform"))
                    .as_str()
                    .unwrap_or("uniform")
                {
                    "distance" => smartcore::neighbors::KNNWeightFunction::Distance,
                    _ => smartcore::neighbors::KNNWeightFunction::Uniform,
                };
                hyperparam_usize!(k, hyperparams, 3);

                let estimator: Option<Box<dyn Estimator>> = match project.task {
                    Task::regression => Some(Box::new(
                        smartcore::neighbors::knn_regressor::KNNRegressor::fit(
                            &x_train,
                            &y_train,
                            smartcore::neighbors::knn_regressor::KNNRegressorParameters::default()
                                .with_algorithm(algorithm)
                                .with_weight(weight)
                                .with_k(k),
                        )
                        .unwrap(),
                    )),

                    Task::classification => Some(Box::new(
                        smartcore::neighbors::knn_classifier::KNNClassifier::fit(
                            &x_train,
                            &y_train,
                            smartcore::neighbors::knn_classifier::KNNClassifierParameters::default(
                            )
                            .with_algorithm(algorithm)
                            .with_weight(weight)
                            .with_k(k),
                        )
                        .unwrap(),
                    )),
                };

                save_estimator!(estimator, self);

                estimator
            }

            Algorithm::random_forest => {
                train_test_split!(dataset, x_train, y_train);

                let max_depth = match hyperparams.get("max_depth") {
                    Some(max_depth) => match max_depth.as_u64() {
                        Some(max_depth) => Some(max_depth as u16),
                        None => None,
                    },
                    None => None,
                };

                let m = match hyperparams.get("m") {
                    Some(m) => match m.as_u64() {
                        Some(m) => Some(m as usize),
                        None => None,
                    },
                    None => None,
                };

                let split_criterion = match hyperparams
                    .get("split_criterion")
                    .unwrap_or(&serde_json::Value::from("gini"))
                    .as_str()
                    .unwrap_or("gini") {
                    "entropy" => smartcore::tree::decision_tree_classifier::SplitCriterion::Entropy,
                    "classification_error" => smartcore::tree::decision_tree_classifier::SplitCriterion::ClassificationError,
                    _ => smartcore::tree::decision_tree_classifier::SplitCriterion::Gini,
                };

                hyperparam_usize!(min_samples_leaf, hyperparams, 1);
                hyperparam_usize!(min_samples_split, hyperparams, 2);
                hyperparam_usize!(n_trees, hyperparams, 10);
                hyperparam_usize!(seed, hyperparams, 0);
                hyperparam_bool!(keep_samples, hyperparams, false);

                let estimator: Option<Box<dyn Estimator>> = match project.task {
                    Task::regression => {
                        let mut params = smartcore::ensemble::random_forest_regressor::RandomForestRegressorParameters::default()
                                    .with_min_samples_leaf(min_samples_leaf)
                                    .with_min_samples_split(min_samples_split)
                                    .with_seed(seed as u64)
                                    .with_n_trees(n_trees as usize)
                                    .with_keep_samples(keep_samples);
                        match max_depth {
                            Some(max_depth) => params = params.with_max_depth(max_depth),
                            None => (),
                        };

                        match m {
                            Some(m) => params = params.with_m(m),
                            None => (),
                        };

                        Some(
                            Box::new(
                                smartcore::ensemble::random_forest_regressor::RandomForestRegressor::fit(
                                    &x_train,
                                    &y_train,
                                    params,
                                ).unwrap()
                            )
                        )
                    }

                    Task::classification => {
                        let mut params = smartcore::ensemble::random_forest_classifier::RandomForestClassifierParameters::default()
                                    .with_min_samples_leaf(min_samples_leaf)
                                    .with_min_samples_split(min_samples_leaf)
                                    .with_seed(seed as u64)
                                    .with_n_trees(n_trees as u16)
                                    .with_keep_samples(keep_samples)
                                    .with_criterion(split_criterion);

                        match max_depth {
                            Some(max_depth) => params = params.with_max_depth(max_depth),
                            None => (),
                        };

                        match m {
                            Some(m) => params = params.with_m(m),
                            None => (),
                        };

                        Some(
                            Box::new(
                                smartcore::ensemble::random_forest_classifier::RandomForestClassifier::fit(
                                    &x_train,
                                    &y_train,
                                    params,
                                ).unwrap()
                            )
                        )
                    }
                };

                save_estimator!(estimator, self);

                estimator
            }
        };
    }

    fn test(&mut self, project: &Project, dataset: &Dataset) {
        let metrics = self.estimator.as_ref().unwrap().test(project.task, dataset);
        self.metrics = Some(JsonB(json!(metrics)));
        Spi::get_one_with_args::<i64>(
            "UPDATE pgml_rust.models SET metrics = $1 WHERE id = $2 RETURNING id",
            vec![
                (
                    PgBuiltInOids::JSONBOID.oid(),
                    JsonB(json!(metrics)).into_datum(),
                ),
                (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
            ],
        )
        .unwrap();
    }
}
