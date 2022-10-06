use std::convert::From;

use linfa::prelude::Predict;
use linfa::traits::Fit;
use ndarray::{ArrayView1, ArrayView2};
use serde::{Deserialize, Serialize};

use super::Bindings;
use crate::orm::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct LinearRegression {
    estimator: linfa_linear::FittedLinearRegression<f32>,
    num_features: usize,
}

impl LinearRegression {
    pub fn fit(dataset: &Dataset, _hyperparams: &Hyperparams) -> Box<dyn Bindings>
    where
        Self: Sized,
    {
        let records = ArrayView2::from_shape(
            (dataset.num_train_rows, dataset.num_features),
            &dataset.x_train,
        )
        .unwrap();

        let targets = ArrayView1::from_shape(dataset.num_train_rows, &dataset.y_train).unwrap();

        let linfa_dataset = linfa::DatasetBase::from((records, targets));
        let estimator = linfa_linear::LinearRegression::default();
        let estimator = estimator.fit(&linfa_dataset).unwrap();

        Box::new(LinearRegression {
            estimator,
            num_features: dataset.num_features,
        })
    }
}

impl Bindings for LinearRegression {
    /// Predict a novel datapoint.
    fn predict(&self, features: &[f32]) -> f32 {
        self.predict_batch(features)[0]
    }

    /// Predict a novel datapoint.
    fn predict_batch(&self, features: &[f32]) -> Vec<f32> {
        let records = ArrayView2::from_shape(
            (features.len() / self.num_features, self.num_features),
            features,
        )
        .unwrap();
        self.estimator.predict(records).targets.into_raw_vec()
    }

    /// Serialize self to bytes
    fn to_bytes(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap()
    }

    /// Deserialize self from bytes, with additional context
    fn from_bytes(bytes: &[u8]) -> Box<dyn Bindings>
    where
        Self: Sized,
    {
        let estimator: LinearRegression = rmp_serde::from_read(bytes).unwrap();
        Box::new(estimator)
    }
}

// #[derive(Debug)]
// pub struct LogisticRegression {
//     estimator: linfa_logistic::FittedLogisticRegression<f32, f32>,
//     num_features: usize,
// }

// impl Bindings for LogisticRegression {
//     fn fit(dataset: &Dataset) -> Box<dyn Bindings> where Self: Sized {
//         let estimator = linfa_logistic::LogisticRegression::default();
//         let estimator = estimator.fit(&dataset.into()).unwrap();

//         Box::new(LogisticRegression { estimator, num_features: dataset.num_features })
//     }

//     /// Predict a novel datapoint.
//     fn predict(&self, features: &[f32]) -> f32 {
//         self.predict_batch(features)[0]
//     }

//     /// Predict a novel datapoint.
//     fn predict_batch(&self, features: &[f32]) -> Vec<f32> {
//         let records = ArrayView2::from_shape(
//             (features.len() / self.num_features, self.num_features),
//             features,
//         )
//         .unwrap();
//         self.estimator.predict(records).targets.into_raw_vec()
//     }

//     /// Deserialize self from bytes, with additional context
//     fn from_bytes(bytes: &[u8]) -> Box<dyn Bindings> where Self: Sized {
//         let estimator: LogisticRegression = rmp_serde::from_read(bytes).unwrap();
//         Box::new(estimator)
//     }

//     /// Serialize self to bytes
//     fn to_bytes(&self) -> Vec<u8> {
//         // Logistic Regression doesn't support Serde (or any kind of ) serialization
//         todo!()
//     }

//     /// The hyperparams used during the fit call
//     fn hyperparams(&self) -> Hyperparams {
//         todo!()
//     }
// }
