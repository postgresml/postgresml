use lightgbm;

use crate::engines::Hyperparams;
use crate::orm::dataset::Dataset;
use crate::orm::estimator::LightgbmBox;
use crate::orm::task::Task;
use serde_json::json;

pub fn lightgbm_train(task: Task, dataset: &Dataset, hyperparams: &Hyperparams) -> LightgbmBox {
    let x_train = dataset.x_train();
    let y_train = dataset.y_train();
    let objective = match task {
        Task::regression => "regression",
        Task::classification => {
            let distinct_labels = dataset.distinct_labels();

            if distinct_labels > 2 {
                "multiclass"
            } else {
                "binary"
            }
        }
    };

    let dataset =
        lightgbm::Dataset::from_vec(x_train, y_train, dataset.num_features as i32).unwrap();

    let bst = lightgbm::Booster::train(
        dataset,
        &json! {{
            "objective": objective,
        }},
    )
    .unwrap();

    LightgbmBox::new(bst)
}

/// Serialize an LightGBm estimator into bytes.
pub fn lightgbm_save(estimator: &LightgbmBox) -> Vec<u8> {
    let r: u64 = rand::random();
    let path = format!("/tmp/pgml_rust_{}.bin", r);

    estimator.save_file(&path).unwrap();

    let bytes = std::fs::read(&path).unwrap();

    std::fs::remove_file(&path).unwrap();

    bytes
}

/// Load an LightGBM estimator from bytes.
pub fn lightgbm_load(data: &Vec<u8>) -> LightgbmBox {
    // Oh boy
    let r: u64 = rand::random();
    let path = format!("/tmp/pgml_rust_{}.bin", r);

    std::fs::write(&path, &data).unwrap();

    let bst = lightgbm::Booster::from_file(&path).unwrap();
    LightgbmBox::new(bst)
}

/// Validate a trained estimator against the test dataset.
pub fn lightgbm_test(estimator: &LightgbmBox, dataset: &Dataset) -> Vec<f32> {
    let x_test = dataset.x_test();
    let num_features = dataset.num_features;

    estimator.predict(&x_test, num_features as i32).unwrap()
}

/// Predict a novel datapoint using the LightGBM estimator.
pub fn lightgbm_predict(estimator: &LightgbmBox, x: &[f32]) -> f32 {
    estimator.predict(&x, x.len() as i32).unwrap()[0]
}
