use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use ndarray::{Array1, Array2};
use once_cell::sync::Lazy;
use pgx::*;
use xgboost::{Booster, DMatrix};

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

    let e: Box<dyn Estimator> = match task {
        Task::regression => match algorithm {
            Algorithm::linear => {
                let estimator: smartcore::linear::linear_regression::LinearRegression<
                    f32,
                    Array2<f32>,
                > = rmp_serde::from_read(&*data).unwrap();
                Box::new(estimator)
            }
            Algorithm::xgboost => {
                let bst = Booster::load_buffer(&*data).unwrap();
                Box::new(BoosterBox::new(bst))
            }
        },
        Task::classification => match algorithm {
            Algorithm::linear => {
                let estimator: smartcore::linear::logistic_regression::LogisticRegression<
                    f32,
                    Array2<f32>,
                > = rmp_serde::from_read(&*data).unwrap();
                Box::new(estimator)
            }
            Algorithm::xgboost => {
                let bst = Booster::load_buffer(&*data).unwrap();
                Box::new(BoosterBox::new(bst))
            }
        },
    };

    let mut estimators = DEPLOYED_ESTIMATORS_BY_PROJECT_NAME.lock().unwrap();
    estimators.insert(name.to_string(), Arc::new(e));
    estimators.get(name).unwrap().clone()
}

fn test_smartcore(
    predictor: &dyn smartcore::api::Predictor<Array2<f32>, Array1<f32>>,
    task: Task,
    dataset: &Dataset,
) -> HashMap<String, f32> {
    let x_test = Array2::from_shape_vec(
        (dataset.num_test_rows, dataset.num_features),
        dataset.x_test().to_vec(),
    )
    .unwrap();
    let y_test = Array1::from_shape_vec(dataset.num_test_rows, dataset.y_test().to_vec()).unwrap();
    let y_hat = smartcore::api::Predictor::predict(predictor, &x_test).unwrap();
    calc_metrics(&y_test, &y_hat, task)
}

fn predict_smartcore(
    predictor: &dyn smartcore::api::Predictor<Array2<f32>, Array1<f32>>,
    features: Vec<f32>,
) -> f32 {
    let features = Array2::from_shape_vec((1, features.len()), features).unwrap();
    smartcore::api::Predictor::predict(predictor, &features).unwrap()[0]
}

fn calc_metrics(y_test: &Array1<f32>, y_hat: &Array1<f32>, task: Task) -> HashMap<String, f32> {
    let mut results = HashMap::new();
    match task {
        Task::regression => {
            results.insert("r2".to_string(), smartcore::metrics::r2(y_test, y_hat));
            results.insert(
                "mean_absolute_error".to_string(),
                smartcore::metrics::mean_absolute_error(y_test, y_hat),
            );
            results.insert(
                "mean_squared_error".to_string(),
                smartcore::metrics::mean_squared_error(y_test, y_hat),
            );
        }
        Task::classification => {
            results.insert(
                "f1".to_string(),
                smartcore::metrics::f1::F1 { beta: 1.0 }.get_score(y_test, y_hat),
            );
            results.insert(
                "precision".to_string(),
                smartcore::metrics::precision(y_test, y_hat),
            );
            results.insert(
                "accuracy".to_string(),
                smartcore::metrics::accuracy(y_test, y_hat),
            );
            results.insert(
                "roc_auc_score".to_string(),
                smartcore::metrics::roc_auc_score(y_test, y_hat),
            );
            results.insert(
                "recall".to_string(),
                smartcore::metrics::recall(y_test, y_hat),
            );
        }
    }
    results
}

#[typetag::serialize(tag = "type")]
pub trait Estimator: Send + Sync + Debug {
    fn test(&self, task: Task, data: &Dataset) -> HashMap<String, f32>;
    fn predict(&self, features: Vec<f32>) -> f32;
}

#[typetag::serialize]
impl Estimator for smartcore::linear::linear_regression::LinearRegression<f32, Array2<f32>> {
    fn test(&self, task: Task, data: &Dataset) -> HashMap<String, f32> {
        test_smartcore(self, task, data)
    }

    fn predict(&self, features: Vec<f32>) -> f32 {
        predict_smartcore(self, features)
    }
}

#[typetag::serialize]
impl Estimator for smartcore::linear::logistic_regression::LogisticRegression<f32, Array2<f32>> {
    fn test(&self, task: Task, data: &Dataset) -> HashMap<String, f32> {
        test_smartcore(self, task, data)
    }

    fn predict(&self, features: Vec<f32>) -> f32 {
        predict_smartcore(self, features)
    }
}

pub struct BoosterBox {
    contents: Box<xgboost::Booster>,
}

impl BoosterBox {
    pub fn new(contents: xgboost::Booster) -> Self {
        BoosterBox {
            contents: Box::new(contents),
        }
    }
}

impl std::ops::Deref for BoosterBox {
    type Target = xgboost::Booster;

    fn deref(&self) -> &Self::Target {
        self.contents.as_ref()
    }
}

impl std::ops::DerefMut for BoosterBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.contents.as_mut()
    }
}

unsafe impl Send for BoosterBox {}
unsafe impl Sync for BoosterBox {}
impl std::fmt::Debug for BoosterBox {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        formatter.debug_struct("BoosterBox").finish()
    }
}
impl serde::Serialize for BoosterBox {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!("this is never hit for now, since we'd need also need a deserializer.")
    }
}

#[typetag::serialize]
impl Estimator for BoosterBox {
    fn test(&self, task: Task, dataset: &Dataset) -> HashMap<String, f32> {
        let mut features = DMatrix::from_dense(dataset.x_test(), dataset.num_test_rows).unwrap();
        features.set_labels(dataset.y_test()).unwrap();
        let y_test =
            Array1::from_shape_vec(dataset.num_test_rows, dataset.y_test().to_vec()).unwrap();
        let y_hat = self.contents.predict(&features).unwrap();
        let y_hat = Array1::from_shape_vec(dataset.num_test_rows, y_hat).unwrap();

        calc_metrics(&y_test, &y_hat, task)
    }

    fn predict(&self, features: Vec<f32>) -> f32 {
        let features = DMatrix::from_dense(&features, 1).unwrap();
        self.contents.predict(&features).unwrap()[0]
    }
}
