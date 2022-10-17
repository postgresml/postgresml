/// Scikit-Learn implementation.
///
/// Scikit needs no introduction. It implements dozens of industry-standard
/// algorithms used in data science and machine learning.
///
/// It uses numpy as its dense matrix.
///
/// Our implementation below calls into Python wrappers
/// defined in `src/bindings/sklearn.py`.
use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::bindings::Bindings;

use crate::orm::*;

pub fn linear_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "linear_regression")
}

pub fn lasso_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "lasso_regression")
}

pub fn svm_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "svm_regression")
}

pub fn elastic_net_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "elastic_net_regression")
}

pub fn ridge_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "ridge_regression")
}

pub fn random_forest_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "random_forest_regression")
}

pub fn xgboost_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "xgboost_regression")
}

pub fn xgboost_random_forest_regression(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "xgboost_random_forest_regression")
}

pub fn orthogonal_matching_persuit_regression(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(
        dataset,
        hyperparams,
        "orthogonal_matching_persuit_regression",
    )
}

pub fn bayesian_ridge_regression(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "bayesian_ridge_regression")
}

pub fn automatic_relevance_determination_regression(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(
        dataset,
        hyperparams,
        "automatic_relevance_determination_regression",
    )
}

pub fn stochastic_gradient_descent_regression(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(
        dataset,
        hyperparams,
        "stochastic_gradient_descent_regression",
    )
}

pub fn passive_aggressive_regression(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "passive_aggressive_regression")
}

pub fn ransac_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "ransac_regression")
}

pub fn theil_sen_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "theil_sen_regression")
}

pub fn huber_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "huber_regression")
}

pub fn quantile_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "quantile_regression")
}

pub fn kernel_ridge_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "kernel_ridge_regression")
}

pub fn gaussian_process_regression(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "gaussian_process_regression")
}

pub fn nu_svm_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "nu_svm_regression")
}

pub fn ada_boost_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "ada_boost_regression")
}

pub fn bagging_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "bagging_regression")
}

pub fn extra_trees_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "extra_trees_regression")
}

pub fn gradient_boosting_trees_regression(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "gradient_boosting_trees_regression")
}

pub fn hist_gradient_boosting_regression(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "hist_gradient_boosting_regression")
}

pub fn least_angle_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "least_angle_regression")
}

pub fn lasso_least_angle_regression(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "lasso_least_angle_regression")
}

pub fn linear_svm_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "linear_svm_regression")
}

pub fn lightgbm_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "lightgbm_regression")
}

pub fn linear_classification(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "linear_classification")
}

pub fn svm_classification(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "svm_classification")
}

pub fn ridge_classification(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "ridge_classification")
}

pub fn random_forest_classification(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "random_forest_classification")
}

pub fn xgboost_classification(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "xgboost_classification")
}

pub fn xgboost_random_forest_classification(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "xgboost_random_forest_classification")
}

pub fn stochastic_gradient_descent_classification(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(
        dataset,
        hyperparams,
        "stochastic_gradient_descent_classification",
    )
}

pub fn perceptron_classification(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "perceptron_classification")
}

pub fn passive_aggressive_classification(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "passive_aggressive_classification")
}

pub fn gaussian_process(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "gaussian_process")
}

pub fn nu_svm_classification(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "nu_svm_classification")
}

pub fn ada_boost_classification(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "ada_boost_classification")
}

pub fn bagging_classification(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "bagging_classification")
}

pub fn extra_trees_classification(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "extra_trees_classification")
}

pub fn gradient_boosting_trees_classification(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(
        dataset,
        hyperparams,
        "gradient_boosting_trees_classification",
    )
}

pub fn hist_gradient_boosting_classification(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(
        dataset,
        hyperparams,
        "hist_gradient_boosting_classification",
    )
}

pub fn linear_svm_classification(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "linear_svm_classification")
}

pub fn lightgbm_classification(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, "lightgbm_classification")
}

fn fit(
    dataset: &Dataset,
    hyperparams: &Hyperparams,
    algorithm_task: &'static str,
) -> Box<dyn Bindings> {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bindings/sklearn.py"
    ));

    let hyperparams = serde_json::to_string(hyperparams).unwrap();

    let (estimator, predict, predict_proba) =
        Python::with_gil(|py| -> (Py<PyAny>, Py<PyAny>, Py<PyAny>) {
            let module = PyModule::from_code(py, module, "", "").unwrap();
            let estimator: Py<PyAny> = module.getattr("estimator").unwrap().into();

            let train: Py<PyAny> = estimator
                .call1(
                    py,
                    PyTuple::new(
                        py,
                        &[
                            String::from(algorithm_task).into_py(py),
                            dataset.num_features.into_py(py),
                            dataset.num_labels.into_py(py),
                            hyperparams.into_py(py),
                        ],
                    ),
                )
                .unwrap();

            let estimator: Py<PyAny> = train
                .call1(py, PyTuple::new(py, &[&dataset.x_train, &dataset.y_train]))
                .unwrap();

            let predict: Py<PyAny> = module
                .getattr("predictor")
                .unwrap()
                .call1(PyTuple::new(py, &[&estimator]))
                .unwrap()
                .extract()
                .unwrap();

            let predict_proba: Py<PyAny> = module
                .getattr("predictor_proba")
                .unwrap()
                .call1(PyTuple::new(py, &[&estimator]))
                .unwrap()
                .extract()
                .unwrap();

            (estimator, predict, predict_proba)
        });

    Box::new(Estimator {
        estimator,
        predict,
        predict_proba,
    })
}

pub struct Estimator {
    estimator: Py<PyAny>,
    predict: Py<PyAny>,
    predict_proba: Py<PyAny>,
}

unsafe impl Send for Estimator {}
unsafe impl Sync for Estimator {}

impl std::fmt::Debug for Estimator {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        formatter.debug_struct("Estimator").finish()
    }
}

impl Bindings for Estimator {
    /// Predict a novel datapoint.
    fn predict(&self, features: &[f32], _num_features: usize, _num_classes: usize) -> Vec<f32> {
        Python::with_gil(|py| -> Vec<f32> {
            self.predict
                .call1(py, PyTuple::new(py, &[features]))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    fn predict_proba(&self, features: &[f32], _num_features: usize) -> Vec<f32> {
        Python::with_gil(|py| -> Vec<f32> {
            self.predict_proba
                .call1(py, PyTuple::new(py, &[features]))
                .unwrap()
                .extract(py)
                .unwrap()
        })
    }

    /// Serialize self to bytes
    fn to_bytes(&self) -> Vec<u8> {
        let module = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/bindings/sklearn.py"
        ));

        Python::with_gil(|py| -> Vec<u8> {
            let module = PyModule::from_code(py, module, "", "").unwrap();
            let save = module.getattr("save").unwrap();
            save.call1(PyTuple::new(py, &[&self.estimator]))
                .unwrap()
                .extract()
                .unwrap()
        })
    }

    /// Deserialize self from bytes, with additional context
    fn from_bytes(bytes: &[u8]) -> Box<dyn Bindings>
    where
        Self: Sized,
    {
        let module = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/bindings/sklearn.py"
        ));

        Python::with_gil(|py| -> Box<dyn Bindings> {
            let module = PyModule::from_code(py, module, "", "").unwrap();
            let load = module.getattr("load").unwrap();
            let estimator: Py<PyAny> = load
                .call1(PyTuple::new(py, &[bytes]))
                .unwrap()
                .extract()
                .unwrap();

            let predict: Py<PyAny> = module
                .getattr("predictor")
                .unwrap()
                .call1(PyTuple::new(py, &[&estimator]))
                .unwrap()
                .extract()
                .unwrap();

            let predict_proba: Py<PyAny> = module
                .getattr("predictor_proba")
                .unwrap()
                .call1(PyTuple::new(py, &[&estimator]))
                .unwrap()
                .extract()
                .unwrap();

            Box::new(Estimator {
                estimator,
                predict,
                predict_proba,
            })
        })
    }
}

fn sklearn_metric(name: &str, ground_truth: &[f32], y_hat: &[f32]) -> f32 {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bindings/sklearn.py"
    ));

    Python::with_gil(|py| -> f32 {
        let module = PyModule::from_code(py, module, "", "").unwrap();
        let calculate_metric = module.getattr("calculate_metric").unwrap();
        let wrapper: Py<PyAny> = calculate_metric
            .call1(PyTuple::new(py, &[name]))
            .unwrap()
            .extract()
            .unwrap();

        let score: f32 = wrapper
            .call1(py, PyTuple::new(py, &[ground_truth, y_hat]))
            .unwrap()
            .extract(py)
            .unwrap();

        score
    })
}

pub fn f1(ground_truth: &[f32], y_hat: &[f32]) -> f32 {
    sklearn_metric("f1", ground_truth, y_hat)
}

pub fn r2(ground_truth: &[f32], y_hat: &[f32]) -> f32 {
    sklearn_metric("r2", ground_truth, y_hat)
}

pub fn precision(ground_truth: &[f32], y_hat: &[f32]) -> f32 {
    sklearn_metric("precision", ground_truth, y_hat)
}

pub fn recall(ground_truth: &[f32], y_hat: &[f32]) -> f32 {
    sklearn_metric("recall", ground_truth, y_hat)
}

pub fn regression_metrics(ground_truth: &[f32], y_hat: &[f32]) -> HashMap<String, f32> {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bindings/sklearn.py"
    ));

    Python::with_gil(|py| -> HashMap<String, f32> {
        let module = PyModule::from_code(py, module, "", "").unwrap();
        let calculate_metric = module.getattr("regression_metrics").unwrap();
        let scores: HashMap<String, f32> = calculate_metric
            .call1(PyTuple::new(py, &[ground_truth, y_hat]))
            .unwrap()
            .extract()
            .unwrap();

        scores
    })
}

pub fn classification_metrics(
    ground_truth: &[f32],
    y_hat: &[f32],
    num_classes: usize,
) -> HashMap<String, f32> {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bindings/sklearn.py"
    ));

    let mut scores = Python::with_gil(|py| -> HashMap<String, f32> {
        let module = PyModule::from_code(py, module, "", "").unwrap();
        let calculate_metric = module.getattr("classification_metrics").unwrap();
        let scores: HashMap<String, f32> = calculate_metric
            .call1(PyTuple::new(py, &[ground_truth, y_hat]))
            .unwrap()
            .extract()
            .unwrap();

        scores
    });

    if num_classes == 2 {
        let roc_auc = sklearn_metric("roc_auc", ground_truth, y_hat);
        scores.insert("roc_auc".to_string(), roc_auc);
    }

    scores
}
