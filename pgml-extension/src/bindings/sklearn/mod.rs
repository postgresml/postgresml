use pgrx::*;
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

use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::{
    bindings::{Bindings, TracebackError},
    create_pymodule,
    orm::*,
};

create_pymodule!("/src/bindings/sklearn/sklearn.py");

macro_rules! wrap_fit {
    ($fn_name:tt, $task:literal) => {
        pub fn $fn_name(dataset: &Dataset, hyperparams: &Hyperparams) -> Result<Box<dyn Bindings>> {
            fit(dataset, hyperparams, $task)
        }
    };
}

wrap_fit!(linear_regression, "linear_regression");
wrap_fit!(lasso_regression, "lasso_regression");
wrap_fit!(svm_regression, "svm_regression");
wrap_fit!(elastic_net_regression, "elastic_net_regression");
wrap_fit!(ridge_regression, "ridge_regression");
wrap_fit!(random_forest_regression, "random_forest_regression");
wrap_fit!(xgboost_regression, "xgboost_regression");
wrap_fit!(xgboost_random_forest_regression, "xgboost_random_forest_regression");
wrap_fit!(
    orthogonal_matching_pursuit_regression,
    "orthogonal_matching_pursuit_regression"
);
wrap_fit!(bayesian_ridge_regression, "bayesian_ridge_regression");
wrap_fit!(
    automatic_relevance_determination_regression,
    "automatic_relevance_determination_regression"
);
wrap_fit!(
    stochastic_gradient_descent_regression,
    "stochastic_gradient_descent_regression"
);
wrap_fit!(passive_aggressive_regression, "passive_aggressive_regression");
wrap_fit!(ransac_regression, "ransac_regression");
wrap_fit!(theil_sen_regression, "theil_sen_regression");
wrap_fit!(huber_regression, "huber_regression");
wrap_fit!(quantile_regression, "quantile_regression");
wrap_fit!(kernel_ridge_regression, "kernel_ridge_regression");
wrap_fit!(gaussian_process_regression, "gaussian_process_regression");
wrap_fit!(nu_svm_regression, "nu_svm_regression");
wrap_fit!(ada_boost_regression, "ada_boost_regression");
wrap_fit!(bagging_regression, "bagging_regression");
wrap_fit!(extra_trees_regression, "extra_trees_regression");
wrap_fit!(gradient_boosting_trees_regression, "gradient_boosting_trees_regression");
wrap_fit!(hist_gradient_boosting_regression, "hist_gradient_boosting_regression");
wrap_fit!(least_angle_regression, "least_angle_regression");
wrap_fit!(lasso_least_angle_regression, "lasso_least_angle_regression");
wrap_fit!(linear_svm_regression, "linear_svm_regression");
wrap_fit!(lightgbm_regression, "lightgbm_regression");
wrap_fit!(catboost_regression, "catboost_regression");
wrap_fit!(linear_classification, "linear_classification");
wrap_fit!(svm_classification, "svm_classification");
wrap_fit!(ridge_classification, "ridge_classification");
wrap_fit!(random_forest_classification, "random_forest_classification");
wrap_fit!(xgboost_classification, "xgboost_classification");
wrap_fit!(
    xgboost_random_forest_classification,
    "xgboost_random_forest_classification"
);
wrap_fit!(
    stochastic_gradient_descent_classification,
    "stochastic_gradient_descent_classification"
);
wrap_fit!(perceptron_classification, "perceptron_classification");
wrap_fit!(passive_aggressive_classification, "passive_aggressive_classification");
wrap_fit!(gaussian_process, "gaussian_process");
wrap_fit!(nu_svm_classification, "nu_svm_classification");
wrap_fit!(ada_boost_classification, "ada_boost_classification");
wrap_fit!(bagging_classification, "bagging_classification");
wrap_fit!(extra_trees_classification, "extra_trees_classification");
wrap_fit!(
    gradient_boosting_trees_classification,
    "gradient_boosting_trees_classification"
);
wrap_fit!(
    hist_gradient_boosting_classification,
    "hist_gradient_boosting_classification"
);
wrap_fit!(linear_svm_classification, "linear_svm_classification");
wrap_fit!(lightgbm_classification, "lightgbm_classification");
wrap_fit!(catboost_classification, "catboost_classification");
wrap_fit!(affinity_propagation, "affinity_propagation_clustering");
wrap_fit!(agglomerative, "agglomerative_clustering");
wrap_fit!(birch, "birch_clustering");
wrap_fit!(dbscan, "dbscan_clustering");
wrap_fit!(feature_agglomeration, "feature_agglomeration_clustering");
wrap_fit!(kmeans, "kmeans_clustering");
wrap_fit!(mini_batch_kmeans, "mini_batch_kmeans_clustering");
wrap_fit!(mean_shift, "mean_shift_clustering");
wrap_fit!(optics, "optics_clustering");
wrap_fit!(spectral, "spectral_clustering");
wrap_fit!(spectral_bi, "spectral_biclustering");
wrap_fit!(spectral_co, "spectral_coclustering");

wrap_fit!(pca, "pca_decomposition");

fn fit(dataset: &Dataset, hyperparams: &Hyperparams, algorithm_task: &'static str) -> Result<Box<dyn Bindings>> {
    let hyperparams = serde_json::to_string(hyperparams).unwrap();

    let (estimator, predict, predict_proba) = Python::with_gil(|py| -> Result<(Py<PyAny>, Py<PyAny>, Py<PyAny>)> {
        let module = get_module!(PY_MODULE);

        let estimator: Py<PyAny> = module.getattr(py, "estimator")?;

        let train: Py<PyAny> = estimator.call1(
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
        )?;

        let estimator: Py<PyAny> = train.call1(py, PyTuple::new(py, [&dataset.x_train, &dataset.y_train]))?;

        let predict: Py<PyAny> = module
            .getattr(py, "predictor")?
            .call1(py, PyTuple::new(py, [&estimator]))?
            .extract(py)?;

        let predict_proba: Py<PyAny> = module
            .getattr(py, "predictor_proba")?
            .call1(py, PyTuple::new(py, [&estimator]))?
            .extract(py)?;

        Ok((estimator, predict, predict_proba))
    })?;

    Ok(Box::new(Estimator {
        estimator,
        predict,
        predict_proba,
    }))
}

pub struct Estimator {
    estimator: Py<PyAny>,
    predict: Py<PyAny>,
    predict_proba: Py<PyAny>,
}

unsafe impl Send for Estimator {}
unsafe impl Sync for Estimator {}

impl std::fmt::Debug for Estimator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        formatter.debug_struct("Estimator").finish()
    }
}

impl Bindings for Estimator {
    /// Predict a novel datapoint.
    fn predict(&self, features: &[f32], _num_features: usize, _num_classes: usize) -> Result<Vec<f32>> {
        Python::with_gil(|py| Ok(self.predict.call1(py, PyTuple::new(py, [features]))?.extract(py)?))
    }

    fn predict_proba(&self, features: &[f32], _num_features: usize) -> Result<Vec<f32>> {
        Python::with_gil(|py| {
            Ok(self
                .predict_proba
                .call1(py, PyTuple::new(py, [features]))?
                .extract(py)?)
        })
    }

    /// Serialize self to bytes
    fn to_bytes(&self) -> Result<Vec<u8>> {
        Python::with_gil(|py| {
            let save = get_module!(PY_MODULE).getattr(py, "save")?;
            Ok(save.call1(py, PyTuple::new(py, [&self.estimator]))?.extract(py)?)
        })
    }

    /// Deserialize self from bytes, with additional context
    fn from_bytes(bytes: &[u8], _hyperparams: &JsonB) -> Result<Box<dyn Bindings>>
    where
        Self: Sized,
    {
        Python::with_gil(|py| -> Result<Box<dyn Bindings>> {
            let module = get_module!(PY_MODULE);

            let load = module.getattr(py, "load")?;
            let estimator: Py<PyAny> = load.call1(py, PyTuple::new(py, [bytes]))?.extract(py)?;

            let predict: Py<PyAny> = module
                .getattr(py, "predictor")?
                .call1(py, PyTuple::new(py, [&estimator]))?
                .extract(py)?;

            let predict_proba: Py<PyAny> = module
                .getattr(py, "predictor_proba")?
                .call1(py, PyTuple::new(py, [&estimator]))?
                .extract(py)?;

            Ok(Box::new(Estimator {
                estimator,
                predict,
                predict_proba,
            }))
        })
    }
}

fn sklearn_metric(name: &str, ground_truth: &[f32], y_hat: &[f32]) -> Result<f32> {
    Python::with_gil(|py| {
        let calculate_metric = get_module!(PY_MODULE).getattr(py, "calculate_metric").unwrap();
        let wrapper: Py<PyAny> = calculate_metric.call1(py, PyTuple::new(py, [name]))?.extract(py)?;

        let score: f32 = wrapper
            .call1(py, PyTuple::new(py, [ground_truth, y_hat]))?
            .extract(py)?;

        Ok(score)
    })
}

pub fn f1(ground_truth: &[f32], y_hat: &[f32]) -> Result<f32> {
    sklearn_metric("f1", ground_truth, y_hat)
}

pub fn r2(ground_truth: &[f32], y_hat: &[f32]) -> Result<f32> {
    sklearn_metric("r2", ground_truth, y_hat)
}

pub fn precision(ground_truth: &[f32], y_hat: &[f32]) -> Result<f32> {
    sklearn_metric("precision", ground_truth, y_hat)
}

pub fn recall(ground_truth: &[f32], y_hat: &[f32]) -> Result<f32> {
    sklearn_metric("recall", ground_truth, y_hat)
}

pub fn confusion_matrix(ground_truth: &[f32], y_hat: &[f32]) -> Result<Vec<Vec<f32>>> {
    Python::with_gil(|py| {
        let calculate_metric = get_module!(PY_MODULE).getattr(py, "calculate_metric")?;
        let wrapper: Py<PyAny> = calculate_metric
            .call1(py, PyTuple::new(py, ["confusion_matrix"]))?
            .extract(py)?;

        let matrix: Vec<Vec<f32>> = wrapper
            .call1(py, PyTuple::new(py, [ground_truth, y_hat]))?
            .extract(py)?;

        Ok(matrix)
    })
}

pub fn regression_metrics(ground_truth: &[f32], y_hat: &[f32]) -> Result<HashMap<String, f32>> {
    Python::with_gil(|py| {
        let calculate_metric = get_module!(PY_MODULE).getattr(py, "regression_metrics")?;
        let scores: HashMap<String, f32> = calculate_metric
            .call1(py, PyTuple::new(py, [ground_truth, y_hat]))?
            .extract(py)?;

        Ok(scores)
    })
}

pub fn classification_metrics(ground_truth: &[f32], y_hat: &[f32], num_classes: usize) -> Result<HashMap<String, f32>> {
    let mut scores = Python::with_gil(|py| -> Result<HashMap<String, f32>> {
        let calculate_metric = get_module!(PY_MODULE).getattr(py, "classification_metrics")?;
        let scores: HashMap<String, f32> = calculate_metric
            .call1(py, PyTuple::new(py, [ground_truth, y_hat]))?
            .extract(py)?;

        Ok(scores)
    })?;

    if num_classes == 2 {
        let roc_auc = sklearn_metric("roc_auc", ground_truth, y_hat)?;
        scores.insert("roc_auc".to_string(), roc_auc);
    }

    Ok(scores)
}

pub fn clustering_metrics(num_features: usize, inputs: &[f32], labels: &[f32]) -> Result<HashMap<String, f32>> {
    Python::with_gil(|py| {
        let calculate_metric = get_module!(PY_MODULE).getattr(py, "clustering_metrics")?;

        let scores: HashMap<String, f32> = calculate_metric
            .call1(py, (num_features, PyTuple::new(py, [inputs, labels])))?
            .extract(py)?;

        Ok(scores)
    })
}

pub fn decomposition_metrics(bindings: &Box<dyn Bindings>) -> Result<HashMap<String, f32>> {
    Python::with_gil(|py| match bindings.as_any().downcast_ref::<Estimator>() {
        Some(estimator) => {
            let calculate_metric = get_module!(PY_MODULE).getattr(py, "decomposition_metrics")?;
            let metrics = calculate_metric.call1(py, PyTuple::new(py, [&estimator.estimator]));
            let metrics = metrics.format_traceback(py)?.extract(py).format_traceback(py)?;
            Ok(metrics)
        }
        None => error!("Can't compute decomposition metrics for bindings other than sklearn"),
    })
}
