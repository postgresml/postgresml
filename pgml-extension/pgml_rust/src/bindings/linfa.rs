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

#[derive(Debug, Serialize, Deserialize)]
pub struct LogisticRegression {
    estimator_binary: Option<linfa_logistic::FittedLogisticRegression<f32, i32>>,
    estimator_multi: Option<linfa_logistic::MultiFittedLogisticRegression<f32, i32>>,
    num_features: usize,
    num_distinct_labels: usize,
}

impl LogisticRegression {
    pub fn fit(dataset: &Dataset, _hyperparams: &Hyperparams) -> Box<dyn Bindings>
    where
        Self: Sized,
    {
        let records = ArrayView2::from_shape(
            (dataset.num_train_rows, dataset.num_features),
            &dataset.x_train,
        )
        .unwrap();

        // Copy to convert to i32 because LogisticRegression doesn't contineous targets.
        let y_train: Vec<i32> = dataset.y_train.iter().map(|x| *x as i32).collect();
        let targets = ArrayView1::from_shape(dataset.num_train_rows, &y_train).unwrap();

        let linfa_dataset = linfa::DatasetBase::from((records, targets));

        if dataset.num_distinct_labels > 2 {
            let estimator = linfa_logistic::MultiLogisticRegression::default();
            let estimator = estimator.fit(&linfa_dataset).unwrap();

            Box::new(LogisticRegression {
                estimator_binary: None,
                estimator_multi: Some(estimator),
                num_features: dataset.num_features,
                num_distinct_labels: dataset.num_distinct_labels,
            })
        } else {
            let estimator = linfa_logistic::LogisticRegression::default();
            let estimator = estimator.fit(&linfa_dataset).unwrap();

            Box::new(LogisticRegression {
                estimator_binary: Some(estimator),
                estimator_multi: None,
                num_features: dataset.num_features,
                num_distinct_labels: dataset.num_distinct_labels,
            })
        }
    }
}

impl Bindings for LogisticRegression {
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

        if self.num_distinct_labels > 2 {
            self.estimator_multi
                .as_ref()
                .unwrap()
                .predict(records)
                .targets
                .into_raw_vec()
                .into_iter()
                .map(|x| x as f32)
                .collect()
        } else {
            self.estimator_binary
                .as_ref()
                .unwrap()
                .predict(records)
                .targets
                .into_raw_vec()
                .into_iter()
                .map(|x| x as f32)
                .collect()
        }
    }

    /// Deserialize self from bytes, with additional context
    fn from_bytes(bytes: &[u8]) -> Box<dyn Bindings>
    where
        Self: Sized,
    {
        let estimator: LogisticRegression = rmp_serde::from_read(bytes).unwrap();
        Box::new(estimator)
    }

    /// Serialize self to bytes
    fn to_bytes(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap()
    }
}
