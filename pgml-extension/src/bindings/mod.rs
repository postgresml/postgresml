use std::fmt::Debug;

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
