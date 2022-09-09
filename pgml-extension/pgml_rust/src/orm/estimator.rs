use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use ndarray::{Array1, Array2};
use once_cell::sync::Lazy;
use pgx::*;
use serde::Serialize;

use crate::orm::Algorithm;
use crate::orm::Dataset;
use crate::orm::Task;

static DEPLOYED_ESTIMATORS_BY_PROJECT_NAME: Lazy<Mutex<HashMap<String, Arc<Box<dyn Estimator>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn find_deployed_estimator_by_project_name(name: &str) -> Arc<Box<dyn Estimator>> {
    {
        let estimators = DEPLOYED_ESTIMATORS_BY_PROJECT_NAME.lock().unwrap();
        let estimator = estimators.get(name);
        if estimator.is_some() {
            return estimator.unwrap().clone();
        }
    }

    let (task, algorithm, data) = Spi::get_three_with_args::<String, String, Vec<u8>>(
        "
        SELECT projects.task::TEXT, models.algorithm::TEXT, files.data
        FROM pgml_rust.files
        JOIN pgml_rust.models
            ON models.id = files.model_id
        JOIN pgml_rust.deployments 
            ON deployments.model_id = models.id
        JOIN pgml_rust.projects
            ON projects.id = deployments.project_id
        WHERE projects.name = $1
        ORDER by deployments.created_at DESC
        LIMIT 1;",
        vec![(PgBuiltInOids::TEXTOID.oid(), name.into_datum())],
    );
    let task = Task::from_str(
        &task.expect(
            format!(
                "Project {} does not have a trained and deployed model.",
                name
            )
            .as_str(),
        ),
    )
    .unwrap();
    let algorithm = Algorithm::from_str(
        &algorithm.expect(
            format!(
                "Project {} does not have a trained and deployed model.",
                name
            )
            .as_str(),
        ),
    )
    .unwrap();
    let data = data.expect(
        format!(
            "Project {} does not have a trained and deployed model.",
            name
        )
        .as_str(),
    );

    let e = match task {
        Task::regression => match algorithm {
            Algorithm::linear => {
                let estimator: smartcore::linear::linear_regression::LinearRegression<
                    f32,
                    Array2<f32>,
                > = rmp_serde::from_read(&*data).unwrap();
                estimator
            }
            Algorithm::xgboost => {
                todo!()
            }
        },
        Task::classification => {
            Algorithm::linear => {
                let estimator: smartcore::linear::logistic_regression::LogisticRegression<
                    f32,
                    Array2<f32>,
                > = rmp_serde::from_read(&*data).unwrap();
                estimator
            }
            Algorithm::xgboost => {
                todo!()
            }        
        }
    };

    let mut estimators = DEPLOYED_ESTIMATORS_BY_PROJECT_NAME.lock().unwrap();
    estimators.insert(name.to_string(), Arc::new(Box::new(e)));
    estimators.get(name).unwrap().clone()
}

#[typetag::serialize(tag = "type")]
pub trait Estimator: Send + Sync + Debug {
    fn test(&self, task: Task, data: &Dataset) -> HashMap<String, f32>;
    fn estimator_predict(&self, features: Vec<f32>) -> f32;
    // fn predict_batch() { todo!() };
}

#[typetag::serialize]
impl<T> Estimator for T
where
    T: smartcore::api::Predictor<Array2<f32>, Array1<f32>> + Serialize + Send + Sync + Debug,
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
                        "mean_absolute_error".to_string(),
                        smartcore::metrics::mean_absolute_error(&y_test, &y_hat),
                    );
                    results.insert(
                        "mean_squared_error".to_string(),
                        smartcore::metrics::mean_squared_error(&y_test, &y_hat),
                    );
                }
                Task::classification => {
                    results.insert(
                        "f1".to_string(),
                        smartcore::metrics::f1::F1 { beta: 1.0 }.get_score(&y_test, &y_hat),
                    );
                    results.insert(
                        "precision".to_string(),
                        smartcore::metrics::precision(&y_test, &y_hat),
                    );
                    results.insert(
                        "accuracy".to_string(),
                        smartcore::metrics::accuracy(&y_test, &y_hat),
                    );
                    results.insert(
                        "roc_auc_score".to_string(),
                        smartcore::metrics::roc_auc_score(&y_test, &y_hat),
                    );
                    results.insert(
                        "recall".to_string(),
                        smartcore::metrics::recall(&y_test, &y_hat),
                    );
                }
            }
        }
        results
    }

    fn estimator_predict(&self, features: Vec<f32>) -> f32 {
        let features = Array2::from_shape_vec((1, features.len()), features).unwrap();
        self.predict(&features).unwrap()[0]
    }
}
