use parking_lot::Mutex;
use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

use ndarray::{Array1, Array2};
use once_cell::sync::Lazy;
use pgx::*;

use crate::bindings::smartcore::{smartcore_predict, smartcore_test};
use crate::bindings::Bindings;

use crate::orm::Algorithm;
use crate::orm::Dataset;
use crate::orm::Runtime;
use crate::orm::Task;

#[allow(clippy::type_complexity)]
static DEPLOYED_ESTIMATORS_BY_MODEL_ID: Lazy<Mutex<HashMap<i64, Arc<Box<dyn Bindings>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Fetch and load the most up-to-date estimator for the given model.
pub fn find_deployed_estimator_by_model_id(model_id: i64) -> Arc<Box<dyn Bindings>> {
    // Get the estimator from process memory, if we already loaded it.
    {
        let estimators = DEPLOYED_ESTIMATORS_BY_MODEL_ID.lock();
        if let Some(estimator) = estimators.get(&model_id) {
            return estimator.clone();
        }
    }

    let (data, runtime, algorithm) = Spi::get_three_with_args::<Vec<u8>, String, String>(
        "SELECT data, runtime::TEXT, algorithm::TEXT FROM pgml.models
        INNER JOIN pgml.files
            ON models.id = files.model_id 
        WHERE models.id = $1
        LIMIT 1",
        vec![(PgBuiltInOids::INT8OID.oid(), model_id.into_datum())],
    );

    let data = data.unwrap_or_else(|| {
        panic!(
            "Project has gone missing for model: {}. Your model store has been corrupted.",
            model_id
        )
    });
    let runtime = Runtime::from_str(&runtime.unwrap()).unwrap();
    let algorithm = Algorithm::from_str(&algorithm.unwrap()).unwrap();

    info!("load {:?} {:?}", runtime, algorithm);
    let bindings: Box<dyn Bindings> = match runtime {
        Runtime::rust => {
            match algorithm {
                Algorithm::xgboost => crate::bindings::xgboost::Estimator::from_bytes(&data),
                Algorithm::lightgbm => crate::bindings::lightgbm::Estimator::from_bytes(&data),
                Algorithm::linear => crate::bindings::linfa::LinearRegression::from_bytes(&data),
                _ => todo!(), //smartcore_load(&data, task, algorithm, &hyperparams),
            }
        }
        Runtime::python => crate::bindings::sklearn::Estimator::from_bytes(&data),
    };

    // Cache the estimator in process memory.
    let mut estimators = DEPLOYED_ESTIMATORS_BY_MODEL_ID.lock();
    estimators.insert(model_id, Arc::new(bindings));
    estimators.get(&model_id).unwrap().clone()
}

/// Caculate model metrics used to evaluate its performance.
pub fn calc_metrics(
    y_test: &Array1<f32>,
    y_hat: &Array1<f32>,
    distinct_labels: usize,
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
