use pgx::*;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use std::collections::HashMap;

use crate::orm::dataset::Dataset;
use crate::orm::estimator::SklearnBox;

#[pg_extern]
pub fn sklearn_version() -> String {
    let mut version = String::new();

    Python::with_gil(|py| {
        let sklearn = py.import("sklearn").unwrap();
        version = sklearn.getattr("__version__").unwrap().extract().unwrap();
    });

    version
}

pub fn sklearn_train(
    algorithm_name: &str,
    dataset: &Dataset,
    hyperparams: HashMap<String, f32>,
) -> SklearnBox {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/backends/wrappers.py"
    ));

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

pub fn sklearn_test(estimator: &SklearnBox, x_test: &[f32], num_features: usize) -> Vec<f32> {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/backends/wrappers.py"
    ));

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
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/backends/wrappers.py"
    ));

    let y_hat: Vec<f32> = Python::with_gil(|py| -> Vec<f32> {
        let module = PyModule::from_code(py, module, "", "").unwrap();
        let predictor = module.getattr("predictor").unwrap();
        let predict = predictor
            .call1(PyTuple::new(
                py,
                &[estimator.contents.as_ref(), &x.len().into_py(py)],
            ))
            .unwrap();

        predict
            .call1(PyTuple::new(py, &[x]))
            .unwrap()
            .extract()
            .unwrap()
    });

    y_hat
}

pub fn sklearn_save(estimator: &SklearnBox) -> Vec<u8> {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/backends/wrappers.py"
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

pub fn sklearn_load(data: &Vec<u8>) -> SklearnBox {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/backends/wrappers.py"
    ));

    Python::with_gil(|py| -> SklearnBox {
        let module = PyModule::from_code(py, module, "", "").unwrap();
        let load = module.getattr("load").unwrap();
        let estimator = load
            .call1(PyTuple::new(py, &[data]))
            .unwrap()
            .extract()
            .unwrap();
        SklearnBox::new(estimator)
    })
}
