//! Rust wrapper around the [XGBoost](https://xgboost.ai) machine learning library.
//!
//! Provides a high level interface for training machine learning models using
//! [gradient boosting](https://en.wikipedia.org/wiki/Gradient_boosting).
//!
//! Currently in the early stages of development, API is likely to be fairly unstable as new
//! features are added.
//!
//! # Basic usage example
//!
//! ```
//! extern crate xgboost;
//!
//! use xgboost::{parameters, DMatrix, Booster};
//!
//! fn main() {
//!     // training matrix with 5 training examples and 3 features
//!     let x_train = &[1.0, 1.0, 1.0,
//!                     1.0, 1.0, 0.0,
//!                     1.0, 1.0, 1.0,
//!                     0.0, 0.0, 0.0,
//!                     1.0, 1.0, 1.0];
//!     let num_rows = 5;
//!     let y_train = &[1.0, 1.0, 1.0, 0.0, 1.0];
//!
//!     // convert training data into XGBoost's matrix format
//!     let mut dtrain = DMatrix::from_dense(x_train, num_rows).unwrap();
//!
//!     // set ground truth labels for the training matrix
//!     dtrain.set_labels(y_train).unwrap();
//!
//!     // test matrix with 1 row
//!     let x_test = &[0.7, 0.9, 0.6];
//!     let num_rows = 1;
//!     let y_test = &[1.0];
//!     let mut dtest = DMatrix::from_dense(x_test, num_rows).unwrap();
//!     dtest.set_labels(y_test).unwrap();
//!
//!     // specify datasets to evaluate against during training
//!     let evaluation_sets = &[(&dtrain, "train"), (&dtest, "test")];
//!
//!     // specify overall training setup
//!     let training_params = parameters::TrainingParametersBuilder::default()
//!         .dtrain(&dtrain)
//!         .evaluation_sets(Some(evaluation_sets))
//!         .build()
//!         .unwrap();
//!
//!     // train model, and print evaluation data
//!     let bst = Booster::train(&training_params).unwrap();
//!
//!     println!("{:?}", bst.predict(&dtest).unwrap());
//! }
//! ```
//!
//! See the [examples](https://github.com/davechallis/rust-xgboost/tree/master/examples) directory for
//! more detailed examples of different features.
//!
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate log;
extern crate xgboost_sys;
extern crate libc;
extern crate tempfile;
extern crate indexmap;

macro_rules! xgb_call {
    ($x:expr) => {
        XGBError::check_return_value(unsafe { $x })
    };
}

mod error;
pub use error::{XGBResult, XGBError};

mod dmatrix;
pub use dmatrix::DMatrix;

mod booster;
pub use booster::{Booster, FeatureMap, FeatureType};
pub mod parameters;
