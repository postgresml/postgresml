//! BoosterParameters for specifying the type of booster that is used when training a model.
//!
//! # Example
//!
//! ```
//! use xgboost::parameters::BoosterParametersBuilder;
//! use xgboost::parameters::BoosterType;
//! use xgboost::parameters::tree::TreeBoosterParametersBuilder;
//!
//! let tree_params = TreeBoosterParametersBuilder::default()
//!     .eta(0.2)
//!     .gamma(3.0)
//!     .subsample(0.75)
//!     .build()
//!     .unwrap();
//! let booster_params = BoosterParametersBuilder::default()
//!     .booster_type(BoosterType::Tree(tree_params))
//!     .build()
//!     .unwrap();
//! ```
use std::default::Default;

use super::{tree, linear, dart};

/// Type of booster to use when training a [Booster](../struct.Booster.html) model.
#[derive(Clone)]
pub enum BoosterType {
    /// Use a tree booster with given parameters when training.
    ///
    /// Construct parameters using
    /// [TreeBoosterParametersBuilder](tree/struct.TreeBoosterParametersBuilder.html).
    Tree(tree::TreeBoosterParameters),

    /// Use a linear booster with given parameters when training.
    ///
    /// Construct parameters using
    /// [LinearBoosterParametersBuilder](linear/struct.LinearBoosterParametersBuilder.html).
    Linear(linear::LinearBoosterParameters),

    /// Use a [DART](https://xgboost.readthedocs.io/en/latest/tutorials/dart.html) booster
    /// with given parameters when training.
    ///
    /// Construct parameters using
    /// [DartBoosterParametersBuilder](dart/struct.DartBoosterParametersBuilder.html).
    Dart(dart::DartBoosterParameters),
}

impl Default for BoosterType {
    fn default() -> Self { BoosterType::Tree(tree::TreeBoosterParameters::default()) }
}

impl BoosterType {
    pub(crate) fn as_string_pairs(&self) -> Vec<(String, String)> {
        match *self {
            BoosterType::Tree(ref p) => p.as_string_pairs(),
            BoosterType::Linear(ref p) => p.as_string_pairs(),
            BoosterType::Dart(ref p) => p.as_string_pairs()
        }
    }
}
