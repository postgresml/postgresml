use lightgbm;

use crate::orm::dataset::Dataset;
use crate::orm::estimator::LightgbmBox;
use crate::orm::task::Task;
use crate::orm::Hyperparams;
use serde_json::json;

pub fn lightgbm_train(task: Task, dataset: &Dataset, hyperparams: &Hyperparams) -> LightgbmBox {
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

    let dataset =
        lightgbm::Dataset::from_vec(x_train, y_train, dataset.num_features as i32).unwrap();

    let bst = lightgbm::Booster::train(dataset, &json! {hyperparams}).unwrap();

    LightgbmBox::new(bst, task)
}

/// Serialize an LightGBm estimator into bytes.
pub fn lightgbm_save(estimator: &LightgbmBox) -> Vec<u8> {
    let r: u64 = rand::random();
    let path = format!("/tmp/pgml_{}.bin", r);

    estimator.save_file(&path).unwrap();

    let bytes = std::fs::read(&path).unwrap();

    std::fs::remove_file(&path).unwrap();

    bytes
}

/// Load an LightGBM estimator from bytes.
pub fn lightgbm_load(data: &Vec<u8>, task: Task) -> LightgbmBox {
    // Oh boy
    let r: u64 = rand::random();
    let path = format!("/tmp/pgml_{}.bin", r);

    std::fs::write(&path, &data).unwrap();

    let bst = lightgbm::Booster::from_file(&path).unwrap();

    std::fs::remove_file(&path).unwrap();

    LightgbmBox::new(bst, task)
}

/// Validate a trained estimator against the test dataset.
pub fn lightgbm_test(estimator: &LightgbmBox, dataset: &Dataset) -> Vec<f32> {
    let x_test = dataset.x_test();
    let num_features = dataset.num_features;

    let results = estimator.predict(x_test, num_features as i32).unwrap();
    let num_class = estimator.num_class().unwrap();

    match estimator.task() {
        // Classification returns probabilities for all classes.
        Task::classification => {
            let y_hat: Vec<f32> = results
                .as_slice()
                .chunks(num_class as usize)
                .map(|chunk| {
                    let mut max = 0.0;
                    let mut answer = 0;
                    for (it, &class_prob) in chunk.iter().enumerate() {
                        if class_prob > max {
                            max = class_prob;
                            answer = it;
                        }
                    }

                    answer as f32
                })
                .collect();

            y_hat
        }

        // Regression returns the predicted value on the curve.
        Task::regression => results.into_iter().map(|y_hat| y_hat as f32).collect(),
    }
}

/// Predict a novel datapoint using the LightGBM estimator.
pub fn lightgbm_predict(estimator: &LightgbmBox, x: &[f32]) -> f32 {
    let results = estimator.predict(x, x.len() as i32).unwrap();

    match estimator.task() {
        Task::classification => {
            let mut max = 0.0;
            let mut answer = 0;
            for (it, &class_prob) in results.iter().enumerate() {
                if class_prob > max {
                    max = class_prob;
                    answer = it;
                }
            }

            answer as f32
        }

        Task::regression => results[0] as f32,
    }
}
