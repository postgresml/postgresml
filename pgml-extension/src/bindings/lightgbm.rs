use crate::bindings::Bindings;
use crate::orm::dataset::Dataset;
use crate::orm::task::Task;
use crate::orm::Hyperparams;
use lightgbm;
use serde_json::json;

pub struct Estimator {
    estimator: lightgbm::Booster,
    num_features: usize,
    num_classes: usize,
}

unsafe impl Send for Estimator {}
unsafe impl Sync for Estimator {}

impl std::fmt::Debug for Estimator {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        formatter.debug_struct("Estimator").finish()
    }
}

pub fn fit_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, Task::regression)
}

pub fn fit_classification(dataset: &Dataset, hyperparams: &Hyperparams) -> Box<dyn Bindings> {
    fit(dataset, hyperparams, Task::classification)
}

fn fit(dataset: &Dataset, hyperparams: &Hyperparams, task: Task) -> Box<dyn Bindings> {
    let mut hyperparams = hyperparams.clone();
    match task {
        Task::regression => {
            hyperparams.insert(
                "objective".to_string(),
                serde_json::Value::from("regression"),
            );
        }
        Task::classification => {
            if dataset.num_distinct_labels > 2 {
                hyperparams.insert(
                    "objective".to_string(),
                    serde_json::Value::from("multiclass"),
                );
                hyperparams.insert(
                    "num_class".to_string(),
                    serde_json::Value::from(dataset.num_distinct_labels),
                );
            } else {
                hyperparams.insert("objective".to_string(), serde_json::Value::from("binary"));
            }
        }
    };

    let data = lightgbm::Dataset::from_vec(
        &dataset.x_train,
        &dataset.y_train,
        dataset.num_features as i32,
    )
    .unwrap();

    let estimator = lightgbm::Booster::train(data, &json! {hyperparams}).unwrap();

    Box::new(Estimator {
        estimator,
        num_features: dataset.num_features,
        num_classes: if task == Task::regression {
            1
        } else {
            dataset.num_distinct_labels
        },
    })
}

impl Bindings for Estimator {
    /// Predict a novel datapoint.
    fn predict(&self, features: &[f32]) -> f32 {
        self.predict_batch(features)[0]
    }

    // Predict the raw probability of classes for a classifier.
    fn predict_proba(&self, features: &[f32]) -> Vec<f32> {
        self.estimator
            .predict(features, self.num_features.try_into().unwrap())
            .unwrap()
            .into_iter()
            .map(|i| i as f32)
            .collect()
    }

    fn predict_joint(&self, _features: &[f32]) -> Vec<f32> {
        todo!("predict_joint is currently only supported by the Python runtime.")
    }

    /// Predict a set of datapoints.
    fn predict_batch(&self, features: &[f32]) -> Vec<f32> {
        let results = self.predict_proba(features);

        // LightGBM returns probabilities for classification. Convert to discrete classes.
        match self.num_classes {
            2 => results.iter().map(|i| i.round()).collect(),
            num_classes if num_classes > 2 => {
                let mut argmax_results = Vec::with_capacity(results.len() / num_classes);
                let mut max_i: usize = 0;
                let mut max_probability = 0.0;
                for (i, &probability) in results.iter().enumerate() {
                    if i % num_classes == 0 && i > 0 {
                        argmax_results.push((max_i % num_classes) as f32);
                        max_probability = 0.0;
                    }
                    if probability > max_probability {
                        max_probability = probability;
                        max_i = i;
                    }
                }
                argmax_results.push((max_i % num_classes) as f32);
                argmax_results
            }
            _ => results,
        }
    }

    /// Serialize self to bytes
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::from((self.num_features as u64).to_be_bytes());
        bytes.append(&mut (self.num_classes as u64).to_be_bytes().to_vec());

        let r: u64 = rand::random();
        let path = format!("/tmp/pgml_{}.bin", r);
        self.estimator.save_file(&path).unwrap();
        bytes.append(&mut std::fs::read(&path).unwrap());
        std::fs::remove_file(&path).unwrap();

        bytes
    }

    /// Deserialize self from bytes, with additional context
    fn from_bytes(bytes: &[u8]) -> Box<dyn Bindings>
    where
        Self: Sized,
    {
        let num_features = u64::from_be_bytes(bytes[..8].try_into().unwrap()) as usize;
        let num_classes = u64::from_be_bytes(bytes[8..16].try_into().unwrap()) as usize;
        let r: u64 = rand::random();
        let path = format!("/tmp/pgml_{}.bin", r);
        std::fs::write(&path, &bytes[16..]).unwrap();
        let estimator = lightgbm::Booster::from_file(&path).unwrap();
        Box::new(Estimator {
            estimator,
            num_features,
            num_classes,
        })
    }
}
