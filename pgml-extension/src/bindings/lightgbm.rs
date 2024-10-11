use crate::bindings::Bindings;
use crate::orm::dataset::Dataset;
use crate::orm::task::Task;
use crate::orm::Hyperparams;

use anyhow::Result;
use lightgbm;
use pgrx::*;
use serde_json::json;

pub struct Estimator {
    estimator: lightgbm::Booster,
}

unsafe impl Send for Estimator {}
unsafe impl Sync for Estimator {}

impl std::fmt::Debug for Estimator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        formatter.debug_struct("Estimator").finish()
    }
}

pub fn fit_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Result<Box<dyn Bindings>> {
    fit(dataset, hyperparams, Task::regression)
}

pub fn fit_classification(dataset: &Dataset, hyperparams: &Hyperparams) -> Result<Box<dyn Bindings>> {
    fit(dataset, hyperparams, Task::classification)
}

fn fit(dataset: &Dataset, hyperparams: &Hyperparams, task: Task) -> Result<Box<dyn Bindings>> {
    let mut hyperparams = hyperparams.clone();
    match task {
        Task::regression => {
            hyperparams.insert("objective".to_string(), serde_json::Value::from("regression"));
        }
        Task::classification => {
            if dataset.num_distinct_labels > 2 {
                hyperparams.insert("objective".to_string(), serde_json::Value::from("multiclass"));
                hyperparams.insert(
                    "num_class".to_string(),
                    serde_json::Value::from(dataset.num_distinct_labels),
                );
            } else {
                hyperparams.insert("objective".to_string(), serde_json::Value::from("binary"));
            }
        }
        _ => error!("lightgbm only supports `regression` and `classification` tasks."),
    };

    let data = lightgbm::Dataset::from_vec(&dataset.x_train, &dataset.y_train, dataset.num_features as i32).unwrap();

    let estimator = lightgbm::Booster::train(data, &json! {hyperparams}).unwrap();

    Ok(Box::new(Estimator { estimator }))
}

impl Bindings for Estimator {
    /// Predict a set of datapoints.
    fn predict(&self, features: &[f32], num_features: usize, num_classes: usize) -> Result<Vec<f32>> {
        let results = self.predict_proba(features, num_features)?;
        Ok(match num_classes {
            // TODO make lightgbm predict both classes like scikit and xgboost
            0 => results,
            2 => results.iter().map(|i| i.round()).collect(),
            _ => results
                .chunks(num_classes)
                .map(|probabilities| {
                    probabilities
                        .iter()
                        .enumerate()
                        .max_by(|(_, a), (_, b)| a.total_cmp(b))
                        .map(|(index, _)| index)
                        .unwrap() as f32
                })
                .collect(),
        })
    }

    // Predict the raw probability of classes for a classifier.
    fn predict_proba(&self, features: &[f32], num_features: usize) -> Result<Vec<f32>> {
        Ok(self
            .estimator
            .predict(features, num_features as i32)?
            .into_iter()
            .map(|i| i as f32)
            .collect())
    }

    /// Serialize self to bytes
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let r: u64 = rand::random();
        let path = format!("/tmp/pgml_{}.bin", r);
        self.estimator.save_file(&path)?;
        let bytes = std::fs::read(&path)?;
        std::fs::remove_file(&path)?;

        Ok(bytes)
    }

    /// Deserialize self from bytes, with additional context
    fn from_bytes(bytes: &[u8], _hyperparams: &JsonB) -> Result<Box<dyn Bindings>>
    where
        Self: Sized,
    {
        let r: u64 = rand::random();
        let path = format!("/tmp/pgml_{}.bin", r);
        std::fs::write(&path, bytes)?;
        let mut estimator = lightgbm::Booster::from_file(&path);
        if estimator.is_err() {
            // backward compatibility w/ 2.0.0
            std::fs::write(&path, &bytes[16..])?;
            estimator = lightgbm::Booster::from_file(&path);
        }
        std::fs::remove_file(&path)?;
        let estimator = estimator?;
        Ok(Box::new(Estimator { estimator }))
    }
}
