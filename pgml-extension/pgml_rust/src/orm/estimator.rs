use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use ndarray::{Array1, Array2};
use once_cell::sync::Lazy;
use pgx::*;
use pyo3::prelude::*;

use crate::engines::sklearn::{sklearn_load, sklearn_predict, sklearn_test};
use crate::engines::smartcore::{smartcore_load, smartcore_predict, smartcore_test};
use crate::engines::xgboost::{xgboost_load, xgboost_predict, xgboost_test};

use crate::engines::engine::Engine;
use crate::orm::Algorithm;
use crate::orm::Dataset;
use crate::orm::Task;

#[allow(clippy::type_complexity)]
static DEPLOYED_ESTIMATORS_BY_MODEL_ID: Lazy<Mutex<HashMap<i64, Arc<Box<dyn Estimator>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Fetch and load the most up-to-date estimator for the given model.
pub fn find_deployed_estimator_by_model_id(model_id: i64) -> Arc<Box<dyn Estimator>> {
    // Get the estimator from process memory, if we already loaded it.
    {
        let estimators = DEPLOYED_ESTIMATORS_BY_MODEL_ID.lock().unwrap();
        if let Some(estimator) = estimators.get(&model_id) {
            return estimator.clone();
        }
    }

    let (task, algorithm) = Spi::get_two_with_args::<String, String>(
        "
        SELECT projects.task::TEXT, models.algorithm::TEXT
        FROM pgml_rust.models
        JOIN pgml_rust.projects
            ON projects.id = models.project_id
        WHERE models.id = $1
        LIMIT 1",
        vec![(PgBuiltInOids::INT8OID.oid(), model_id.into_datum())],
    );

    let task = Task::from_str(&task.unwrap_or_else(|| {
        panic!(
            "Project has gone missing for model: {}. Your model store has been corrupted.",
            model_id
        )
    }))
    .unwrap();

    let algorithm = Algorithm::from_str(&algorithm.unwrap_or_else(|| {
        panic!(
            "Project has gone missing for model: {}. Your model store has been corrupted.",
            model_id
        )
    }))
    .unwrap();

    let (data, hyperparams, engine) = Spi::get_three_with_args::<Vec<u8>, JsonB, String>(
        "SELECT data, hyperparams, engine::TEXT FROM pgml_rust.models
        INNER JOIN pgml_rust.files
            ON models.id = files.model_id 
        WHERE models.id = $1
        LIMIT 1",
        vec![(PgBuiltInOids::INT8OID.oid(), model_id.into_datum())],
    );

    let hyperparams: &serde_json::Value = &hyperparams.unwrap().0;
    let hyperparams = hyperparams.as_object().unwrap();

    let data = data.unwrap_or_else(|| {
        panic!(
            "Project has gone missing for model: {}. Your model store has been corrupted.",
            model_id
        )
    });

    let engine = Engine::from_str(&engine.unwrap()).unwrap();

    let estimator: Box<dyn Estimator> = match engine {
        Engine::xgboost => Box::new(xgboost_load(&data)),
        Engine::smartcore => smartcore_load(&data, task, algorithm, &hyperparams),
        Engine::sklearn => Box::new(sklearn_load(&data)),
        _ => todo!(),
    };

    // Cache the estimator in process memory.
    let mut estimators = DEPLOYED_ESTIMATORS_BY_MODEL_ID.lock().unwrap();
    estimators.insert(model_id, Arc::new(estimator));
    estimators.get(&model_id).unwrap().clone()
}

/// Caculate model metrics used to evaluate its performance.
fn calc_metrics(
    y_test: &Array1<f32>,
    y_hat: &Array1<f32>,
    distinct_labels: u32,
    task: Task,
) -> HashMap<String, f32> {
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
                "recall".to_string(),
                smartcore::metrics::recall(y_test, y_hat),
            );
            results.insert(
                "accuracy".to_string(),
                smartcore::metrics::accuracy(y_test, y_hat),
            );
            if distinct_labels == 2 {
                results.insert(
                    "roc_auc_score".to_string(),
                    smartcore::metrics::roc_auc_score(y_test, y_hat),
                );
            }
        }
    }

    results
}

/// The estimator trait that has to be implemented by all
/// algorithms we use in PostgresML.
#[typetag::serialize(tag = "type")]
pub trait Estimator: Send + Sync + Debug {
    /// Validate the algorithm agains the test dataset.
    fn test(&self, task: Task, data: &Dataset) -> HashMap<String, f32>;

    /// Predict a novel datapoint.
    fn predict(&self, features: Vec<f32>) -> f32;
}

/// Implement the Estimator trait (it's always the same)
/// for all supported algorithms.
macro_rules! smartcore_estimator_impl {
    ($estimator:ty) => {
        #[typetag::serialize]
        impl Estimator for $estimator {
            fn test(&self, task: Task, dataset: &Dataset) -> HashMap<String, f32> {
                let y_hat = smartcore_test(self, dataset);
                let y_test =
                    Array1::from_shape_vec(dataset.num_test_rows, dataset.y_test().to_vec())
                        .unwrap();

                calc_metrics(&y_test, &y_hat, dataset.distinct_labels(), task)
            }

            fn predict(&self, features: Vec<f32>) -> f32 {
                smartcore_predict(self, features)
            }
        }
    };
}

smartcore_estimator_impl!(smartcore::linear::linear_regression::LinearRegression<f32, Array2<f32>>);
smartcore_estimator_impl!(smartcore::linear::logistic_regression::LogisticRegression<f32, Array2<f32>>);
smartcore_estimator_impl!(smartcore::svm::svc::SVC<f32, Array2<f32>, smartcore::svm::LinearKernel>);
smartcore_estimator_impl!(smartcore::svm::svr::SVR<f32, Array2<f32>, smartcore::svm::LinearKernel>);
smartcore_estimator_impl!(smartcore::svm::svc::SVC<f32, Array2<f32>, smartcore::svm::SigmoidKernel<f32>>);
smartcore_estimator_impl!(smartcore::svm::svr::SVR<f32, Array2<f32>, smartcore::svm::SigmoidKernel<f32>>);
smartcore_estimator_impl!(smartcore::svm::svc::SVC<f32, Array2<f32>, smartcore::svm::PolynomialKernel<f32>>);
smartcore_estimator_impl!(smartcore::svm::svr::SVR<f32, Array2<f32>, smartcore::svm::PolynomialKernel<f32>>);
smartcore_estimator_impl!(smartcore::svm::svc::SVC<f32, Array2<f32>, smartcore::svm::RBFKernel<f32>>);
smartcore_estimator_impl!(smartcore::svm::svr::SVR<f32, Array2<f32>, smartcore::svm::RBFKernel<f32>>);
smartcore_estimator_impl!(smartcore::linear::lasso::Lasso<f32, Array2<f32>>);
smartcore_estimator_impl!(smartcore::linear::elastic_net::ElasticNet<f32, Array2<f32>>);
smartcore_estimator_impl!(smartcore::linear::ridge_regression::RidgeRegression<f32, Array2<f32>>);
smartcore_estimator_impl!(smartcore::neighbors::knn_regressor::KNNRegressor<f32, smartcore::math::distance::euclidian::Euclidian>);
smartcore_estimator_impl!(smartcore::neighbors::knn_classifier::KNNClassifier<f32, smartcore::math::distance::euclidian::Euclidian>);
smartcore_estimator_impl!(smartcore::ensemble::random_forest_regressor::RandomForestRegressor<f32>);
smartcore_estimator_impl!(
    smartcore::ensemble::random_forest_classifier::RandomForestClassifier<f32>
);

/// XGBoost implementation of the Estimator trait.
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
        panic!("This is not used because we don't use Serde to serialize or deserialize XGBoost, it comes with its own.")
    }
}

#[typetag::serialize]
impl Estimator for BoosterBox {
    fn test(&self, task: Task, dataset: &Dataset) -> HashMap<String, f32> {
        let y_hat =
            Array1::from_shape_vec(dataset.num_test_rows, xgboost_test(self, dataset)).unwrap();
        let y_test =
            Array1::from_shape_vec(dataset.num_test_rows, dataset.y_test().to_vec()).unwrap();

        calc_metrics(&y_test, &y_hat, dataset.distinct_labels(), task)
    }

    fn predict(&self, features: Vec<f32>) -> f32 {
        xgboost_predict(self, &features)
    }
}

/// A wrapper around a Scikit estimator.
/// The estimator is a Python object and can only be used
/// inside Python::with_gil.
pub struct SklearnBox {
    pub contents: Box<Py<PyAny>>,
}

impl Drop for SklearnBox {
    fn drop(&mut self) {
        // I don't think this works because drop for self must
        // be executed before the drop of fields?
        Python::with_gil(|_py| {});
    }
}

impl SklearnBox {
    pub fn new(contents: Py<PyAny>) -> SklearnBox {
        SklearnBox {
            contents: Box::new(contents),
        }
    }
}

impl std::ops::Deref for SklearnBox {
    type Target = Py<PyAny>;

    fn deref(&self) -> &Self::Target {
        self.contents.as_ref()
    }
}

impl std::ops::DerefMut for SklearnBox {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.contents.as_mut()
    }
}

unsafe impl Send for SklearnBox {}
unsafe impl Sync for SklearnBox {}

impl std::fmt::Debug for SklearnBox {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        formatter.debug_struct("SklearnBox").finish()
    }
}

impl serde::Serialize for SklearnBox {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        panic!("We don't use Serde for Scikit-Learn models. Scikit is using pickle in Python instead for backwards compatibility.")
    }
}

#[typetag::serialize]
impl Estimator for SklearnBox {
    fn test(&self, task: Task, dataset: &Dataset) -> HashMap<String, f32> {
        let y_test = dataset.y_test();
        let y_hat = sklearn_test(&self, dataset);

        calc_metrics(
            &Array1::from_shape_vec(dataset.num_test_rows, y_test.to_vec()).unwrap(),
            &Array1::from_shape_vec(dataset.num_test_rows, y_hat).unwrap(),
            dataset.distinct_labels(),
            task,
        )
    }

    fn predict(&self, features: Vec<f32>) -> f32 {
        let score = sklearn_predict(self, &features);
        score[0]
    }
}
