use lightgbm;
use crate::bindings::Bindings;
use crate::orm::dataset::Dataset;
use crate::orm::task::Task;
use crate::orm::Hyperparams;
use serde_json::json;

pub struct Estimator {
    estimator: lightgbm::Booster,
    num_features: usize,
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
    let x_train = dataset.x_train();
    let y_train = dataset.y_train();
    let mut hyperparams = hyperparams.clone();
    match task {
        Task::regression => {
            hyperparams.insert(
                "objective".to_string(),
                serde_json::Value::from("regression"),
            );
        }
        Task::classification => {
            let distinct_labels = dataset.distinct_labels();

            if distinct_labels > 2 {
                hyperparams.insert(
                    "objective".to_string(),
                    serde_json::Value::from("multiclass"),
                );
                hyperparams.insert(
                    "num_class".to_string(),
                    serde_json::Value::from(distinct_labels),
                ); // [0, num_class)
            } else {
                hyperparams.insert("objective".to_string(), serde_json::Value::from("binary"));
            }
        }
    };

    let data =
        lightgbm::Dataset::from_vec(x_train, y_train, dataset.num_features as i32).unwrap();

    let estimator = lightgbm::Booster::train(data, &json! {hyperparams}).unwrap();

    Box::new(Estimator { 
        estimator, 
        num_features: dataset.num_features, 
    })
}

impl Bindings for Estimator {
    /// Predict a novel datapoint.
    fn predict(&self, features: &[f32]) -> f32 {
        self.predict_batch(features)[0]
    }

    /// Predict a novel datapoint.
    fn predict_batch(&self, features: &[f32]) -> Vec<f32> {
        let results = self
            .estimator
            .predict(features, self.num_features.try_into().unwrap())
            .unwrap();
        results.into_iter().map(|i| i as f32).collect()
        // TODO handle multiclass
        //     match estimator.task() {
        //         Task::classification => {
        //             let mut max = 0.0;
        //             let mut answer = 0;
        //             for (it, &class_prob) in results.iter().enumerate() {
        //                 if class_prob > max {
        //                     max = class_prob;
        //                     answer = it;
        //                 }
        //             }

        //             answer as f32
        //         }

        //         Task::regression => results[0] as f32,
        //     }
    }

    /// Serialize self to bytes
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::from((self.num_features as u64).to_be_bytes());

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
        let r: u64 = rand::random();
        let path = format!("/tmp/pgml_{}.bin", r);
        std::fs::write(&path, &bytes[8..]).unwrap();
        let estimator = lightgbm::Booster::from_file(&path).unwrap();
        Box::new(Estimator {
            estimator,
            num_features,
        })
    }
}
