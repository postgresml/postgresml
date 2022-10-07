pub mod lightgbm;
pub mod linfa;
pub mod sklearn;
pub mod smartcore;
pub mod transformers;
pub mod xgboost;

use crate::orm::*;

pub type Fit = fn(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings>;

/// The Bindings trait that has to be implemented by all algorithm
/// providers we use in PostgresML. We don't rely on Serde serialization,
/// since scikit-learn estimators were originally serialized in pure Python as
/// pickled objects, and neither xgboost or linfa estimators completely
/// implement serde.
pub trait Bindings: Send + Sync {
    /// Predict a novel datapoint.
    fn predict(&self, features: &[f32]) -> f32;

    /// Predict a set of datapoints.
    fn predict_batch(&self, features: &[f32]) -> Vec<f32>;

    /// Deserialize self from bytes, with additional context
    fn from_bytes(bytes: &[u8]) -> Box<dyn Bindings>
    where
        Self: Sized;

    /// Serialize self to bytes
    fn to_bytes(&self) -> Vec<u8>;
}
