pub mod linear_regression;

/// SmartCore implementation.
///
/// SmartCore is a replacement for Scikit-Learn written in pure Rust.
/// It is not yet complete, but has a good list of algorithms we can
/// use already.
///
/// It uses ndarray for as its dense matrix.
use crate::orm::algorithm::Algorithm;
use crate::orm::dataset::Dataset;
use crate::orm::estimator::Estimator;
use crate::orm::task::Task;

use crate::engines::FromJSON;

use ndarray::{Array1, Array2};

use rmp_serde;
use serde_json;
use smartcore;

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

/// Train a SmartCore estimator.
#[allow(non_snake_case)]
pub fn smartcore_train(
    task: Task,
    algorithm: Algorithm,
    dataset: &Dataset,
    hyperparams: &serde_json::Map<std::string::String, serde_json::Value>,
) -> Box<dyn Estimator> {
    let x_train = Array2::from_shape_vec(
        (dataset.num_train_rows, dataset.num_features),
        dataset.x_train().to_vec(),
    )
    .unwrap();

    let y_train =
        Array1::from_shape_vec(dataset.num_train_rows, dataset.y_train().to_vec()).unwrap();

    match algorithm {
        // Support Vector Machine
        // The smartcore SVM algorithm doesn't handle errors yet,
        // so be careful passing in parameters that you don't know very well.
        Algorithm::svm => {
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

            match task {
                Task::regression => match hyperparams.get("kernel") {
                    Some(kernel) => match kernel.as_str().unwrap_or("linear") {
                        "poly" => Box::new(
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
                        ),

                        "sigmoid" => Box::new(
                            smartcore::svm::svr::SVR::fit(
                                &x_train,
                                &y_train,
                                smartcore::svm::svr::SVRParameters::default()
                                    .with_eps(eps)
                                    .with_c(C)
                                    .with_tol(tol)
                                    .with_kernel(smartcore::svm::Kernels::sigmoid(gamma, coef0)),
                            )
                            .unwrap(),
                        ),

                        "rbf" => Box::new(
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
                        ),

                        _ => Box::new(
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
                        ),
                    },

                    None => Box::new(
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
                    ),
                },

                Task::classification => match hyperparams.get("kernel") {
                    Some(kernel) => match kernel.as_str().unwrap_or("linear") {
                        "poly" => Box::new(
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
                        ),

                        "sigmoid" => Box::new(
                            smartcore::svm::svc::SVC::fit(
                                &x_train,
                                &y_train,
                                smartcore::svm::svc::SVCParameters::default()
                                    .with_epoch(epoch)
                                    .with_c(C)
                                    .with_tol(tol)
                                    .with_kernel(smartcore::svm::Kernels::sigmoid(gamma, coef0)),
                            )
                            .unwrap(),
                        ),

                        "rbf" => Box::new(
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
                        ),

                        _ => Box::new(
                            smartcore::svm::svc::SVC::fit(
                                &x_train,
                                &y_train,
                                smartcore::svm::svc::SVCParameters::default()
                                    .with_epoch(epoch)
                                    .with_tol(tol)
                                    .with_kernel(smartcore::svm::Kernels::linear()),
                            )
                            .unwrap(),
                        ),
                    },

                    None => Box::new(
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
                    ),
                },
            }
        }

        Algorithm::linear => match task {
            Task::regression => {
                Box::new(
                    smartcore::linear::linear_regression::LinearRegression::fit(
                        &x_train, &y_train, smartcore::linear::linear_regression::LinearRegressionParameters::from_json(hyperparams).unwrap(),
                    )
                    .unwrap(),
                )
            }

            Task::classification => {
                todo!();
            }
        },

        Algorithm::xgboost => panic!("SmartCore does not support XGBoost"),

        Algorithm::lasso => {
            hyperparam_f32!(alpha, hyperparams, 1.0);
            hyperparam_bool!(normalize, hyperparams, false);
            hyperparam_f32!(tol, hyperparams, 1e-4);
            hyperparam_usize!(max_iter, hyperparams, 1000);

            match task {
                Task::regression => Box::new(
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
                ),

                Task::classification => panic!("SmartCore Lasso does not support classification"),
            }
        }

        Algorithm::elastic_net => {
            hyperparam_f32!(alpha, hyperparams, 1.0);
            hyperparam_f32!(l1_ratio, hyperparams, 0.5);
            hyperparam_bool!(normalize, hyperparams, false);
            hyperparam_f32!(tol, hyperparams, 1e-4);
            hyperparam_usize!(max_iter, hyperparams, 1000);

            match task {
                Task::regression => Box::new(
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
                ),

                Task::classification => {
                    panic!("SmartCore Elastic Net does not support classification")
                }
            }
        }

        Algorithm::ridge => {
            hyperparam_f32!(alpha, hyperparams, 1.0);
            hyperparam_bool!(normalize, hyperparams, false);

            let solver = match hyperparams.get("solver") {
                Some(solver) => match solver.as_str().unwrap_or("cholesky") {
                    "svd" => smartcore::linear::ridge_regression::RidgeRegressionSolverName::SVD,
                    _ => smartcore::linear::ridge_regression::RidgeRegressionSolverName::Cholesky,
                },
                None => smartcore::linear::ridge_regression::RidgeRegressionSolverName::SVD,
            };

            match task {
                Task::regression => Box::new(
                    smartcore::linear::ridge_regression::RidgeRegression::fit(
                        &x_train,
                        &y_train,
                        smartcore::linear::ridge_regression::RidgeRegressionParameters::default()
                            .with_alpha(alpha)
                            .with_normalize(normalize)
                            .with_solver(solver),
                    )
                    .unwrap(),
                ),

                Task::classification => panic!("SmartCore Ridge does not support classification"),
            }
        }

        Algorithm::kmeans => todo!(),
        Algorithm::dbscan => todo!(),

        Algorithm::knn => {
            let algorithm = match hyperparams
                .get("algorithm")
                .unwrap_or(&serde_json::Value::from("linear_search"))
                .as_str()
                .unwrap_or("linear_search")
            {
                "cover_tree" => smartcore::algorithm::neighbour::KNNAlgorithmName::CoverTree,
                _ => smartcore::algorithm::neighbour::KNNAlgorithmName::LinearSearch,
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

            match task {
                Task::regression => Box::new(
                    smartcore::neighbors::knn_regressor::KNNRegressor::fit(
                        &x_train,
                        &y_train,
                        smartcore::neighbors::knn_regressor::KNNRegressorParameters::default()
                            .with_algorithm(algorithm)
                            .with_weight(weight)
                            .with_k(k),
                    )
                    .unwrap(),
                ),

                Task::classification => Box::new(
                    smartcore::neighbors::knn_classifier::KNNClassifier::fit(
                        &x_train,
                        &y_train,
                        smartcore::neighbors::knn_classifier::KNNClassifierParameters::default()
                            .with_algorithm(algorithm)
                            .with_weight(weight)
                            .with_k(k),
                    )
                    .unwrap(),
                ),
            }
        }

        Algorithm::random_forest => {
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
                .unwrap_or("gini")
            {
                "entropy" => smartcore::tree::decision_tree_classifier::SplitCriterion::Entropy,
                "classification_error" => {
                    smartcore::tree::decision_tree_classifier::SplitCriterion::ClassificationError
                }
                _ => smartcore::tree::decision_tree_classifier::SplitCriterion::Gini,
            };

            hyperparam_usize!(min_samples_leaf, hyperparams, 1);
            hyperparam_usize!(min_samples_split, hyperparams, 2);
            hyperparam_usize!(n_trees, hyperparams, 10);
            hyperparam_usize!(seed, hyperparams, 0);
            hyperparam_bool!(keep_samples, hyperparams, false);

            match task {
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

                    Box::new(
                        smartcore::ensemble::random_forest_regressor::RandomForestRegressor::fit(
                            &x_train, &y_train, params,
                        )
                        .unwrap(),
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

                    Box::new(
                        smartcore::ensemble::random_forest_classifier::RandomForestClassifier::fit(
                            &x_train, &y_train, params,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }
}

/// Save a SmartCore estimator.
pub fn smartcore_save(estimator: &Box<dyn Estimator>) -> Vec<u8> {
    let bytes: Vec<u8> = rmp_serde::to_vec(&*estimator).unwrap();
    bytes
}

/// Load an SmartCore estimator from bytes.
pub fn smartcore_load(
    data: &[u8],
    task: Task,
    algorithm: Algorithm,
    hyperparams: &serde_json::Map<std::string::String, serde_json::Value>,
) -> Box<dyn Estimator> {
    match task {
        Task::regression => match algorithm {
            Algorithm::linear => {
                let estimator: smartcore::linear::linear_regression::LinearRegression<
                    f32,
                    Array2<f32>,
                > = rmp_serde::from_read(data).unwrap();
                Box::new(estimator)
            }
            Algorithm::lasso => {
                let estimator: smartcore::linear::lasso::Lasso<f32, Array2<f32>> =
                    rmp_serde::from_read(data).unwrap();
                Box::new(estimator)
            }
            Algorithm::elastic_net => {
                let estimator: smartcore::linear::elastic_net::ElasticNet<f32, Array2<f32>> =
                    rmp_serde::from_read(data).unwrap();
                Box::new(estimator)
            }
            Algorithm::ridge => {
                let estimator: smartcore::linear::ridge_regression::RidgeRegression<
                    f32,
                    Array2<f32>,
                > = rmp_serde::from_read(data).unwrap();
                Box::new(estimator)
            }

            Algorithm::kmeans => todo!(),
            Algorithm::dbscan => todo!(),

            Algorithm::knn => {
                let estimator: smartcore::neighbors::knn_regressor::KNNRegressor<
                    f32,
                    smartcore::math::distance::euclidian::Euclidian,
                > = rmp_serde::from_read(data).unwrap();
                Box::new(estimator)
            }

            Algorithm::random_forest => {
                let estimator: smartcore::ensemble::random_forest_regressor::RandomForestRegressor<
                    f32,
                > = rmp_serde::from_read(data).unwrap();
                Box::new(estimator)
            }

            Algorithm::xgboost => panic!("SmartCore does not support XGBoost"),

            Algorithm::svm => match hyperparams.get("kernel") {
                Some(kernel) => match kernel.as_str().unwrap_or("linear") {
                    "poly" => {
                        let estimator: smartcore::svm::svr::SVR<
                            f32,
                            Array2<f32>,
                            smartcore::svm::PolynomialKernel<f32>,
                        > = rmp_serde::from_read(data).unwrap();
                        Box::new(estimator)
                    }

                    "sigmoid" => {
                        let estimator: smartcore::svm::svr::SVR<
                            f32,
                            Array2<f32>,
                            smartcore::svm::SigmoidKernel<f32>,
                        > = rmp_serde::from_read(data).unwrap();
                        Box::new(estimator)
                    }

                    "rbf" => {
                        let estimator: smartcore::svm::svr::SVR<
                            f32,
                            Array2<f32>,
                            smartcore::svm::RBFKernel<f32>,
                        > = rmp_serde::from_read(data).unwrap();
                        Box::new(estimator)
                    }

                    _ => {
                        let estimator: smartcore::svm::svr::SVR<
                            f32,
                            Array2<f32>,
                            smartcore::svm::LinearKernel,
                        > = rmp_serde::from_read(data).unwrap();
                        Box::new(estimator)
                    }
                },

                None => {
                    let estimator: smartcore::svm::svr::SVR<
                        f32,
                        Array2<f32>,
                        smartcore::svm::LinearKernel,
                    > = rmp_serde::from_read(data).unwrap();
                    Box::new(estimator)
                }
            },
        },

        Task::classification => match algorithm {
            Algorithm::linear => {
                todo!();
            }

            Algorithm::lasso => panic!("SmartCore Lasso does not support classification"),
            Algorithm::elastic_net => {
                panic!("SmartCore Elastic Net does not support classification")
            }
            Algorithm::ridge => panic!("SmartCore Ridge does not support classification"),
            Algorithm::kmeans => todo!(),
            Algorithm::dbscan => todo!(),

            Algorithm::knn => {
                let estimator: smartcore::neighbors::knn_classifier::KNNClassifier<
                    f32,
                    smartcore::math::distance::euclidian::Euclidian,
                > = rmp_serde::from_read(data).unwrap();
                Box::new(estimator)
            }

            Algorithm::random_forest => {
                let estimator: smartcore::ensemble::random_forest_classifier::RandomForestClassifier<f32> =
                    rmp_serde::from_read(data).unwrap();
                Box::new(estimator)
            }

            Algorithm::xgboost => panic!("SmartCore does not support XGBoost"),

            Algorithm::svm => match &hyperparams.get("kernel") {
                Some(kernel) => match kernel.as_str().unwrap_or("linear") {
                    "poly" => {
                        let estimator: smartcore::svm::svc::SVC<
                            f32,
                            Array2<f32>,
                            smartcore::svm::PolynomialKernel<f32>,
                        > = rmp_serde::from_read(data).unwrap();
                        Box::new(estimator)
                    }

                    "sigmoid" => {
                        let estimator: smartcore::svm::svc::SVC<
                            f32,
                            Array2<f32>,
                            smartcore::svm::SigmoidKernel<f32>,
                        > = rmp_serde::from_read(data).unwrap();
                        Box::new(estimator)
                    }

                    "rbf" => {
                        let estimator: smartcore::svm::svc::SVC<
                            f32,
                            Array2<f32>,
                            smartcore::svm::RBFKernel<f32>,
                        > = rmp_serde::from_read(data).unwrap();
                        Box::new(estimator)
                    }

                    _ => {
                        let estimator: smartcore::svm::svc::SVC<
                            f32,
                            Array2<f32>,
                            smartcore::svm::LinearKernel,
                        > = rmp_serde::from_read(data).unwrap();
                        Box::new(estimator)
                    }
                },

                None => {
                    let estimator: smartcore::svm::svc::SVC<
                        f32,
                        Array2<f32>,
                        smartcore::svm::LinearKernel,
                    > = rmp_serde::from_read(data).unwrap();
                    Box::new(estimator)
                }
            },
        },
    }
}

/// Validate a trained estimator against the test dataset.
pub fn smartcore_test(
    estimator: &dyn smartcore::api::Predictor<Array2<f32>, Array1<f32>>,
    dataset: &Dataset,
) -> Array1<f32> {
    let x_test = Array2::from_shape_vec(
        (dataset.num_test_rows, dataset.num_features),
        dataset.x_test().to_vec(),
    )
    .unwrap();

    smartcore::api::Predictor::predict(estimator, &x_test).unwrap()
}

/// Predict a novel datapoint using the SmartCore estimator.
pub fn smartcore_predict(
    predictor: &dyn smartcore::api::Predictor<Array2<f32>, Array1<f32>>,
    features: Vec<f32>,
) -> f32 {
    let features = Array2::from_shape_vec((1, features.len()), features).unwrap();
    smartcore::api::Predictor::predict(predictor, &features).unwrap()[0]
}

// /// Predict a novel datapoint using the SmartCore estimator.
// pub fn smartcore_predict(estimator: &Box<dyn Estimator>, x: &[f32]) -> f32 {
//     let x = DMatrix::from_dense(x, 1).unwrap();
//     estimator.predict(&x).unwrap()[0]
// }
