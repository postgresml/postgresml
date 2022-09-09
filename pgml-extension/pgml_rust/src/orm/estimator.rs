use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use ndarray::{Array1, Array2};
use once_cell::sync::Lazy;
use serde::Serialize;

use crate::orm::Dataset;
use crate::orm::Task;

static DEPLOYED_ESTIMATORS_BY_MODEL_ID: Lazy<Mutex<HashMap<i64, Arc<dyn Estimator>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[typetag::serialize(tag = "type")]
pub trait Estimator: Send + Sync {
    fn find_deployed(model_id: i64) -> Box<dyn Estimator> where Self: Sized {
        todo!()
    }
    fn test(&self, task: Task, data: &Dataset) -> HashMap<String, f32>;
    fn estimator_predict(&self, features: Vec<f32>) -> f32;
    // fn predict_batch();
}

#[typetag::serialize]
impl<T> Estimator for T
where
    T: smartcore::api::Predictor<Array2<f32>, Array1<f32>> + Serialize + Send + Sync,
{
    fn test(&self, task: Task, dataset: &Dataset) -> HashMap<String, f32> {
        let x_test = Array2::from_shape_vec(
            (dataset.num_test_rows, dataset.num_features),
            dataset.x_test().to_vec(),
        )
        .unwrap();
        let y_hat = self.predict(&x_test).unwrap();
        let mut results = HashMap::new();
        if dataset.num_labels == 1 {
            let y_test =
                Array1::from_shape_vec(dataset.num_test_rows, dataset.y_test().to_vec()).unwrap();
            match task {
                Task::regression => {
                    results.insert("r2".to_string(), smartcore::metrics::r2(&y_test, &y_hat));
                    results.insert(
                        "mse".to_string(),
                        smartcore::metrics::mean_squared_error(&y_test, &y_hat),
                    );
                }
                Task::classification => todo!(),
            }
        }
        results
    }

    fn estimator_predict(&self, features: Vec<f32>) -> f32 {
        let features = Array2::from_shape_vec((features.len(), 1), features).unwrap();
        self.predict(&features).unwrap()[0]
    }
}

