use std::fmt::Debug;

use anyhow::{anyhow, Result};
#[allow(unused_imports)] // used for test macros
use pgrx::*;
use pyo3::{pyfunction, PyResult, Python};

use crate::orm::*;

#[pyfunction]
pub fn r_insert_logs(project_id: i64, model_id: i64, logs: String) -> PyResult<String> {
    let id_value = Spi::get_one_with_args::<i64>(
        "INSERT INTO pgml.logs (project_id, model_id, logs) VALUES ($1, $2, $3::JSONB) RETURNING id;",
        vec![
            (PgBuiltInOids::INT8OID.oid(), project_id.into_datum()),
            (PgBuiltInOids::INT8OID.oid(), model_id.into_datum()),
            (PgBuiltInOids::TEXTOID.oid(), logs.into_datum()),
        ],
    )
    .unwrap()
    .unwrap();
    Ok(format!("Inserted logs with id: {}", id_value))
}

#[pyfunction]
pub fn r_log(level: String, message: String) -> PyResult<String> {
    match level.as_str() {
        "info" => info!("{}", message),
        "warning" => warning!("{}", message),
        "debug" => debug1!("{}", message),
        "error" => error!("{}", message),
        _ => info!("{}", message),
    };
    Ok(message)
}

#[cfg(feature = "python")]
#[macro_export]
macro_rules! create_pymodule {
    ($pyfile:literal) => {
        pub static PY_MODULE: once_cell::sync::Lazy<anyhow::Result<pyo3::Py<pyo3::types::PyModule>>> =
            once_cell::sync::Lazy::new(|| {
                pyo3::Python::with_gil(|py| -> anyhow::Result<pyo3::Py<pyo3::types::PyModule>> {
                    use $crate::bindings::TracebackError;
                    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), $pyfile));
                    let module = pyo3::types::PyModule::from_code(py, src, "transformers.py", "__main__")
                        .format_traceback(py)?;
                    module.add_function(wrap_pyfunction!($crate::bindings::r_insert_logs, module)?)?;
                    module.add_function(wrap_pyfunction!($crate::bindings::r_log, module)?)?;
                    Ok(module.into())
                })
            });
    };
}

#[cfg(feature = "python")]
#[macro_export]
macro_rules! get_module {
    ($module:ident) => {
        match $module.as_ref() {
            Ok(module) => module,
            Err(e) => anyhow::bail!(e),
        }
    };
}

#[cfg(feature = "python")]
pub mod langchain;
pub mod lightgbm;
pub mod linfa;
#[cfg(feature = "python")]
pub mod python;
#[cfg(feature = "python")]
pub mod sklearn;
#[cfg(feature = "python")]
pub mod transformers;
pub mod xgboost;

pub type Fit = fn(dataset: &Dataset, hyperparams: &Hyperparams) -> Result<Box<dyn Bindings>>;

use std::any::Any;

pub trait AToAny: 'static {
    fn as_any(&self) -> &dyn Any;
}

impl<T: 'static> AToAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The Bindings trait that has to be implemented by all algorithm
/// providers we use in PostgresML. We don't rely on Serde serialization,
/// since scikit-learn estimators were originally serialized in pure Python as
/// pickled objects, and neither xgboost nor linfa estimators completely
/// implement serde.
pub trait Bindings: Send + Sync + Debug + AToAny {
    /// Predict a set of datapoints.
    fn predict(&self, features: &[f32], num_features: usize, num_classes: usize) -> Result<Vec<f32>>;

    /// Predict the probability of each class.
    fn predict_proba(&self, features: &[f32], num_features: usize) -> Result<Vec<f32>>;

    /// Serialize self to bytes
    fn to_bytes(&self) -> Result<Vec<u8>>;

    /// Deserialize self from bytes, with additional context
    fn from_bytes(bytes: &[u8], _hyperparams: &JsonB) -> Result<Box<dyn Bindings>>
    where
        Self: Sized;
}

pub trait TracebackError<T> {
    fn format_traceback(self, py: Python<'_>) -> Result<T>;
}

impl<T> TracebackError<T> for PyResult<T> {
    fn format_traceback(self, py: Python<'_>) -> Result<T> {
        self.map_err(|e| match e.traceback(py) {
            Some(traceback) => match traceback.format() {
                Ok(traceback) => anyhow!("{traceback} {e}"),
                Err(format_e) => anyhow!("{e} {format_e}"),
            },
            None => anyhow!("{e}"),
        })
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn test_linear_algorithms() {
        dataset::load_diabetes(None);
        dataset::load_breast_cancer(None);
        let regression = Project::create("regression", Task::regression);
        let mut diabetes = Snapshot::create(
            "pgml.diabetes",
            Some(vec!["target".to_string()]),
            0.5,
            Sampling::last,
            false,
            JsonB(serde_json::Value::Object(Hyperparams::new())),
        );
        let classification = Project::create("classification", Task::classification);
        let mut breast_cancer = Snapshot::create(
            "pgml.breast_cancer",
            Some(vec!["malignant".to_string()]),
            0.5,
            Sampling::last,
            false,
            JsonB(serde_json::Value::Object(Hyperparams::new())),
        );

        let mut regressors = Vec::new();
        let mut classifiers = Vec::new();
        for algorithm in [Algorithm::linear, Algorithm::xgboost, Algorithm::lightgbm] {
            regressors.extend([Runtime::python, Runtime::rust].map(|runtime| {
                let model = Model::create(
                    &regression,
                    &mut diabetes,
                    algorithm,
                    JsonB(serde_json::Value::Object(Hyperparams::new())),
                    None,
                    JsonB(serde_json::Value::Object(Hyperparams::new())),
                    JsonB(serde_json::Value::Object(Hyperparams::new())),
                    Some(runtime),
                );
                println!(
                    "regressor runtime:{:?} fit: {:?} evaluate: {:?} score: {:?}",
                    runtime,
                    model.fit_time(),
                    model.score_time(),
                    model.r2()
                );
                model
            }));

            classifiers.extend([Runtime::python, Runtime::rust].map(|runtime| {
                let model = Model::create(
                    &classification,
                    &mut breast_cancer,
                    algorithm,
                    JsonB(serde_json::Value::Object(Hyperparams::new())),
                    None,
                    JsonB(serde_json::Value::Object(Hyperparams::new())),
                    JsonB(serde_json::Value::Object(Hyperparams::new())),
                    Some(runtime),
                );
                println!(
                    "classifier runtime:{:?} fit: {:?} evaluate: {:?} score: {:?}",
                    runtime,
                    model.fit_time(),
                    model.score_time(),
                    model.f1(),
                );
                model
            }));
        }

        for chunk in regressors
            .into_iter()
            .zip(classifiers)
            .flat_map(|s| [s.0, s.1])
            .collect::<Vec<Model>>()
            .chunks(4)
        {
            let (python_regressor, python_classifier, rust_regressor, rust_classifier) =
                (&chunk[0], &chunk[1], &chunk[2], &chunk[3]);
            println!("{:?}", python_regressor.algorithm);
            println!("objective,training,evaluating,score");
            println!(
                "{:?},{:?},{:?},{:?}",
                python_regressor.project.task,
                (python_regressor.fit_time() / rust_regressor.fit_time()),
                (python_regressor.score_time() / rust_regressor.score_time()),
                (rust_regressor.r2() - python_regressor.r2()),
            );
            println!(
                "{:?},{:?},{:?},{:?}",
                python_classifier.project.task,
                (python_classifier.fit_time() / rust_classifier.fit_time()),
                (python_classifier.score_time() / rust_classifier.score_time()),
                (rust_classifier.f1() - python_classifier.f1()),
            );
        }
    }
}
