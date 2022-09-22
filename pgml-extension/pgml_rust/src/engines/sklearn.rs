/// Scikit-Learn implementation.
///
/// Scikit needs no introduction. It implements dozens of industry-standard
/// algorithms used in data science and machine learning.
///
/// It uses numpy as its dense matrix.
///
/// Our implementation below calls into Python wrappers
/// defined in `src/engines/wrappers.py`.
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::engines::Hyperparams;
use crate::orm::algorithm::Algorithm;
use crate::orm::dataset::Dataset;
use crate::orm::estimator::SklearnBox;
use crate::orm::search::Search;
use crate::orm::task::Task;

use pgx::*;

#[pg_extern]
pub fn sklearn_version() -> String {
    let mut version = String::new();

    Python::with_gil(|py| {
        let sklearn = py.import("sklearn").unwrap();
        version = sklearn.getattr("__version__").unwrap().extract().unwrap();
    });

    version
}

fn sklearn_algorithm_name(task: Task, algorithm: Algorithm) -> &'static str {
    match task {
        Task::regression => match algorithm {
            Algorithm::linear => "linear_regression",
            Algorithm::lasso => "lasso_regression",
            Algorithm::svm => "svm_regression",
            Algorithm::elastic_net => "elastic_net_regression",
            Algorithm::ridge => "ridge_regression",
            Algorithm::random_forest => "random_forest_regression",
            Algorithm::xgboost => "xgboost_regression",
            Algorithm::xgboost_random_forest => "xgboost_random_forest_regression",
            Algorithm::orthogonal_matching_pursuit => "orthogonal_matching_persuit_regression",
            Algorithm::bayesian_ridge => "bayesian_ridge_regression",
            Algorithm::automatic_relevance_determination => {
                "automatic_relevance_determination_regression"
            }
            Algorithm::stochastic_gradient_descent => "stochastic_gradient_descent_regression",
            Algorithm::passive_aggressive => "passive_aggressive_regression",
            Algorithm::ransac => "ransac_regression",
            Algorithm::theil_sen => "theil_sen_regression",
            Algorithm::huber => "huber_regression",
            Algorithm::quantile => "quantile_regression",
            Algorithm::kernel_ridge => "kernel_ridge_regression",
            Algorithm::gaussian_process => "gaussian_process_regression",
            Algorithm::nu_svm => "nu_svm_regression",
            Algorithm::ada_boost => "ada_boost_regression",
            Algorithm::bagging => "bagging_regression",
            Algorithm::extra_trees => "extra_trees_regression",
            Algorithm::gradient_boosting_trees => "gradient_boosting_trees_regression",
            Algorithm::hist_gradient_boosting => "hist_gradient_boosting_regression",
            Algorithm::least_angle => "least_angle_regression",
            Algorithm::lasso_least_angle => "lasso_least_angle_regression",
            Algorithm::linear_svm => "linear_svm_regression",
            _ => panic!("{:?} does not support regression", algorithm),
        },

        Task::classification => match algorithm {
            Algorithm::linear => "linear_classification",
            Algorithm::svm => "svm_classification",
            Algorithm::ridge => "ridge_classification",
            Algorithm::random_forest => "random_forest_classification",
            Algorithm::xgboost => "xgboost_classification",
            Algorithm::xgboost_random_forest => "xgboost_random_forest_classification",
            Algorithm::stochastic_gradient_descent => "stochastic_gradient_descent_classification",
            Algorithm::perceptron => "perceptron_classification",
            Algorithm::passive_aggressive => "passive_aggressive_classification",
            Algorithm::gaussian_process => "gaussian_process",
            Algorithm::nu_svm => "nu_svm_classification",
            Algorithm::ada_boost => "ada_boost_classification",
            Algorithm::bagging => "bagging_classification",
            Algorithm::extra_trees => "extra_trees_classification",
            Algorithm::gradient_boosting_trees => "gradient_boosting_trees_classification",
            Algorithm::hist_gradient_boosting => "hist_gradient_boosting_classification",
            Algorithm::linear_svm => "linear_svm_classification",
            _ => panic!("{:?} does not support classification", algorithm),
        },
    }
}

pub fn sklearn_train(
    task: Task,
    algorithm: Algorithm,
    dataset: &Dataset,
    hyperparams: &serde_json::Map<std::string::String, serde_json::Value>,
) -> SklearnBox {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engines/wrappers.py"
    ));

    let algorithm_name = sklearn_algorithm_name(task, algorithm);
    let hyperparams = serde_json::to_string(hyperparams).unwrap();

    let estimator = Python::with_gil(|py| -> Py<PyAny> {
        let module = PyModule::from_code(py, module, "", "").unwrap();
        let estimator: Py<PyAny> = module.getattr("estimator").unwrap().into();

        let train: Py<PyAny> = estimator
            .call1(
                py,
                PyTuple::new(
                    py,
                    &[
                        String::from(algorithm_name).into_py(py),
                        dataset.num_features.into_py(py),
                        hyperparams.into_py(py),
                    ],
                ),
            )
            .unwrap();

        train
            .call1(
                py,
                PyTuple::new(py, &[dataset.x_train(), dataset.y_train()]),
            )
            .unwrap()
    });

    SklearnBox::new(estimator)
}

pub fn sklearn_test(estimator: &SklearnBox, dataset: &Dataset) -> Vec<f32> {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engines/wrappers.py"
    ));

    let x_test = dataset.x_test();
    let num_features = dataset.num_features;

    let y_hat: Vec<f32> = Python::with_gil(|py| -> Vec<f32> {
        let module = PyModule::from_code(py, module, "", "").unwrap();
        let predictor = module.getattr("predictor").unwrap();
        let predict = predictor
            .call1(PyTuple::new(
                py,
                &[estimator.contents.as_ref(), &num_features.into_py(py)],
            ))
            .unwrap();

        predict
            .call1(PyTuple::new(py, &[x_test]))
            .unwrap()
            .extract()
            .unwrap()
    });

    y_hat
}

pub fn sklearn_predict(estimator: &SklearnBox, x: &[f32]) -> Vec<f32> {
    let y_hat: Vec<f32> = Python::with_gil(|py| -> Vec<f32> {
        estimator
            .contents
            .call1(py, PyTuple::new(py, &[x]))
            .unwrap()
            .extract(py)
            .unwrap()
    });

    y_hat
}

pub fn sklearn_save(estimator: &SklearnBox) -> Vec<u8> {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engines/wrappers.py"
    ));

    Python::with_gil(|py| -> Vec<u8> {
        let module = PyModule::from_code(py, module, "", "").unwrap();
        let save = module.getattr("save").unwrap();
        save.call1(PyTuple::new(py, &[estimator.contents.as_ref()]))
            .unwrap()
            .extract()
            .unwrap()
    })
}

pub fn sklearn_load(data: &Vec<u8>, num_features: i32) -> SklearnBox {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engines/wrappers.py"
    ));

    Python::with_gil(|py| -> SklearnBox {
        let module = PyModule::from_code(py, module, "", "").unwrap();
        let load = module.getattr("load").unwrap();
        let estimator = load
            .call1(PyTuple::new(py, &[data]))
            .unwrap()
            .extract()
            .unwrap();
        let predict = module.getattr("predictor").unwrap();
        let estimator = predict
            .call1(PyTuple::new(py, &[estimator, num_features.into_py(py)]))
            .unwrap()
            .extract()
            .unwrap();

        SklearnBox::new(estimator)
    })
}

/// Hyperparameter search using Scikit's
/// RandomizedSearchCV or GridSearchCV.
pub fn sklearn_search(
    task: Task,
    algorithm: Algorithm,
    search: Search,
    dataset: &Dataset,
    hyperparams: &Hyperparams,
    search_params: &Hyperparams,
) -> (SklearnBox, Hyperparams) {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engines/wrappers.py"
    ));

    let algorithm_name = sklearn_algorithm_name(task, algorithm);

    Python::with_gil(|py| -> (SklearnBox, Hyperparams) {
        let module = PyModule::from_code(py, module, "", "").unwrap();
        let estimator_search = module.getattr("estimator_search").unwrap();
        let train = estimator_search
            .call1(PyTuple::new(
                py,
                &[
                    algorithm_name.into_py(py),
                    dataset.num_features.into_py(py),
                    serde_json::to_string(hyperparams).unwrap().into_py(py),
                    serde_json::to_string(search_params).unwrap().into_py(py),
                    search.to_string().into_py(py),
                    None::<String>.into_py(py),
                ],
            ))
            .unwrap();

        let (estimator, hyperparams): (Py<PyAny>, String) = train
            .call1(PyTuple::new(py, &[dataset.x_train(), dataset.y_train()]))
            .unwrap()
            .extract()
            .unwrap();

        let estimator = SklearnBox::new(estimator);
        let hyperparams: Hyperparams = serde_json::from_str::<Hyperparams>(&hyperparams).unwrap();

        (estimator, hyperparams)
    })
}
