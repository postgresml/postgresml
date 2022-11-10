use std::fmt::Debug;

#[allow(unused_imports)] // used for test macros
use pgx::*;

use crate::orm::*;

pub mod lightgbm;
pub mod linfa;
#[cfg(feature = "python")]
pub mod sklearn;
#[cfg(feature = "python")]
pub mod transformers;
pub mod xgboost;

pub type Fit = fn(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings>;

/// The Bindings trait that has to be implemented by all algorithm
/// providers we use in PostgresML. We don't rely on Serde serialization,
/// since scikit-learn estimators were originally serialized in pure Python as
/// pickled objects, and neither xgboost or linfa estimators completely
/// implement serde.
pub trait Bindings: Send + Sync + Debug {
    /// Predict a set of datapoints.
    fn predict(&self, features: &[f32], num_features: usize, num_classes: usize) -> Vec<f32>;

    /// Predict the probability of each class.
    fn predict_proba(&self, features: &[f32], num_features: usize) -> Vec<f32>;

    /// Serialize self to bytes
    fn to_bytes(&self) -> Vec<u8>;

    /// Deserialize self from bytes, with additional context
    fn from_bytes(bytes: &[u8]) -> Box<dyn Bindings>
    where
        Self: Sized;
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
            vec!["target".to_string()],
            0.5,
            Sampling::last,
            false,
        );
        let classification = Project::create("classification", Task::classification);
        let mut breast_cancer = Snapshot::create(
            "pgml.breast_cancer",
            vec!["malignant".to_string()],
            0.5,
            Sampling::last,
            false,
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
