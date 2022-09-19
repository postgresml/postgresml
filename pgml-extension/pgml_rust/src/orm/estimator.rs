use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use ndarray::{Array1, Array2};
use once_cell::sync::Lazy;
use pgx::*;
use pyo3::prelude::*;
use xgboost::{Booster, DMatrix};

use crate::backends::sklearn::{sklearn_load, sklearn_predict, sklearn_test};
use crate::orm::Algorithm;
use crate::orm::Dataset;
use crate::orm::Task;

#[allow(clippy::type_complexity)]
static DEPLOYED_ESTIMATORS_BY_PROJECT_NAME: Lazy<Mutex<HashMap<String, Arc<Box<dyn Estimator>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn find_deployed_estimator_by_project_name(name: &str) -> Arc<Box<dyn Estimator>> {
    {
        let estimators = DEPLOYED_ESTIMATORS_BY_PROJECT_NAME.lock().unwrap();
        let estimator = estimators.get(name);
        if let Some(estimator) = estimator {
            return estimator.clone();
        }
    }

    let (task, algorithm, model_id) = Spi::get_three_with_args::<String, String, i64>(
        "
        SELECT projects.task::TEXT, models.algorithm::TEXT, models.id AS model_id
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
    let task = Task::from_str(&task.unwrap_or_else(|| {
        panic!(
            "Project {} does not have a trained and deployed model.",
            name
        )
    }))
    .unwrap();
    let algorithm = Algorithm::from_str(&algorithm.unwrap_or_else(|| {
        panic!(
            "Project {} does not have a trained and deployed model.",
            name
        )
    }))
    .unwrap();

    let (data, hyperparams) = Spi::get_two_with_args::<Vec<u8>, JsonB>(
        "SELECT data, hyperparams FROM pgml_rust.models
        INNER JOIN pgml_rust.files
        ON models.id = files.model_id WHERE models.id = $1
        LIMIT 1",
        vec![(PgBuiltInOids::INT8OID.oid(), model_id.into_datum())],
    );

    let hyperparams = hyperparams.unwrap();

    let data = data.unwrap_or_else(|| {
        panic!(
            "Project {} does not have a trained and deployed model.",
            name
        )
    });

    let e: Box<dyn Estimator> = match task {
        Task::regression => match algorithm {
            Algorithm::linear => {
                let estimator: smartcore::linear::linear_regression::LinearRegression<
                    f32,
                    Array2<f32>,
                > = rmp_serde::from_read(&*data).unwrap();
                Box::new(estimator)
            }
            Algorithm::lasso => {
                let estimator: smartcore::linear::lasso::Lasso<f32, Array2<f32>> =
                    rmp_serde::from_read(&*data).unwrap();
                Box::new(estimator)
            }
            Algorithm::elastic_net => {
                let estimator: smartcore::linear::elastic_net::ElasticNet<f32, Array2<f32>> =
                    rmp_serde::from_read(&*data).unwrap();
                Box::new(estimator)
            }
            Algorithm::ridge => {
                let estimator: smartcore::linear::ridge_regression::RidgeRegression<
                    f32,
                    Array2<f32>,
                > = rmp_serde::from_read(&*data).unwrap();
                Box::new(estimator)
            }
            Algorithm::kmeans => todo!(),

            Algorithm::dbscan => todo!(),

            Algorithm::knn => {
                let estimator: smartcore::neighbors::knn_regressor::KNNRegressor<
                    f32,
                    smartcore::math::distance::euclidian::Euclidian,
                > = rmp_serde::from_read(&*data).unwrap();
                Box::new(estimator)
            }

            Algorithm::random_forest => {
                let estimator: smartcore::ensemble::random_forest_regressor::RandomForestRegressor<
                    f32,
                > = rmp_serde::from_read(&*data).unwrap();
                Box::new(estimator)
            }

            Algorithm::xgboost => {
                let bst = Booster::load_buffer(&*data).unwrap();
                Box::new(BoosterBox::new(bst))
            }
            Algorithm::svm => match &hyperparams.0.as_object().unwrap().get("kernel") {
                Some(kernel) => match kernel.as_str().unwrap_or("linear") {
                    "poly" => {
                        let estimator: smartcore::svm::svr::SVR<
                            f32,
                            Array2<f32>,
                            smartcore::svm::PolynomialKernel<f32>,
                        > = rmp_serde::from_read(&*data).unwrap();
                        Box::new(estimator)
                    }

                    "sigmoid" => {
                        let estimator: smartcore::svm::svr::SVR<
                            f32,
                            Array2<f32>,
                            smartcore::svm::SigmoidKernel<f32>,
                        > = rmp_serde::from_read(&*data).unwrap();
                        Box::new(estimator)
                    }

                    "rbf" => {
                        let estimator: smartcore::svm::svr::SVR<
                            f32,
                            Array2<f32>,
                            smartcore::svm::RBFKernel<f32>,
                        > = rmp_serde::from_read(&*data).unwrap();
                        Box::new(estimator)
                    }

                    _ => {
                        let estimator: smartcore::svm::svr::SVR<
                            f32,
                            Array2<f32>,
                            smartcore::svm::LinearKernel,
                        > = rmp_serde::from_read(&*data).unwrap();
                        Box::new(estimator)
                    }
                },

                None => {
                    let estimator: smartcore::svm::svr::SVR<
                        f32,
                        Array2<f32>,
                        smartcore::svm::LinearKernel,
                    > = rmp_serde::from_read(&*data).unwrap();
                    Box::new(estimator)
                }
            },
        },
        Task::classification => match algorithm {
            Algorithm::linear => {
                // TODO: check backend and support both backends here.
                let bytes: Vec<u8> = rmp_serde::from_read(&*data).unwrap();
                let estimator = sklearn_load(&bytes);
                Box::new(estimator)
            }
            Algorithm::lasso => panic!("Lasso does not support classification"),
            Algorithm::elastic_net => panic!("Elastic Net does not support classification"),
            Algorithm::ridge => panic!("Ridge does not support classification"),

            Algorithm::kmeans => todo!(),

            Algorithm::dbscan => todo!(),

            Algorithm::knn => {
                let estimator: smartcore::neighbors::knn_classifier::KNNClassifier<
                    f32,
                    smartcore::math::distance::euclidian::Euclidian,
                > = rmp_serde::from_read(&*data).unwrap();
                Box::new(estimator)
            }

            Algorithm::random_forest => {
                let estimator: smartcore::ensemble::random_forest_classifier::RandomForestClassifier<f32> =
                    rmp_serde::from_read(&*data).unwrap();
                Box::new(estimator)
            }

            Algorithm::xgboost => {
                let bst = Booster::load_buffer(&*data).unwrap();
                Box::new(BoosterBox::new(bst))
            }
            Algorithm::svm => match &hyperparams.0.as_object().unwrap().get("kernel") {
                Some(kernel) => match kernel.as_str().unwrap_or("linear") {
                    "poly" => {
                        let estimator: smartcore::svm::svc::SVC<
                            f32,
                            Array2<f32>,
                            smartcore::svm::PolynomialKernel<f32>,
                        > = rmp_serde::from_read(&*data).unwrap();
                        Box::new(estimator)
                    }

                    "sigmoid" => {
                        let estimator: smartcore::svm::svc::SVC<
                            f32,
                            Array2<f32>,
                            smartcore::svm::SigmoidKernel<f32>,
                        > = rmp_serde::from_read(&*data).unwrap();
                        Box::new(estimator)
                    }

                    "rbf" => {
                        let estimator: smartcore::svm::svc::SVC<
                            f32,
                            Array2<f32>,
                            smartcore::svm::RBFKernel<f32>,
                        > = rmp_serde::from_read(&*data).unwrap();
                        Box::new(estimator)
                    }

                    _ => {
                        let estimator: smartcore::svm::svc::SVC<
                            f32,
                            Array2<f32>,
                            smartcore::svm::LinearKernel,
                        > = rmp_serde::from_read(&*data).unwrap();
                        Box::new(estimator)
                    }
                },

                None => {
                    let estimator: smartcore::svm::svc::SVC<
                        f32,
                        Array2<f32>,
                        smartcore::svm::LinearKernel,
                    > = rmp_serde::from_read(&*data).unwrap();
                    Box::new(estimator)
                }
            },
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
    calc_metrics(&y_test, &y_hat, dataset.distinct_labels(), task)
}

fn predict_smartcore(
    predictor: &dyn smartcore::api::Predictor<Array2<f32>, Array1<f32>>,
    features: Vec<f32>,
) -> f32 {
    let features = Array2::from_shape_vec((1, features.len()), features).unwrap();
    smartcore::api::Predictor::predict(predictor, &features).unwrap()[0]
}

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

#[typetag::serialize(tag = "type")]
pub trait Estimator: Send + Sync + Debug {
    fn test(&self, task: Task, data: &Dataset) -> HashMap<String, f32>;
    fn predict(&self, features: Vec<f32>) -> f32;
}

/// Implement the Estimator trait (it's always the same)
/// for all supported algorithms.
macro_rules! smartcore_estimator_impl {
    ($estimator:ty) => {
        #[typetag::serialize]
        impl Estimator for $estimator {
            fn test(&self, task: Task, data: &Dataset) -> HashMap<String, f32> {
                test_smartcore(self, task, data)
            }

            fn predict(&self, features: Vec<f32>) -> f32 {
                predict_smartcore(self, features)
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
        calc_metrics(&y_test, &y_hat, dataset.distinct_labels(), task)
    }

    fn predict(&self, features: Vec<f32>) -> f32 {
        let features = DMatrix::from_dense(&features, 1).unwrap();
        self.contents.predict(&features).unwrap()[0]
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
        unreachable!()
    }
}

#[typetag::serialize]
impl Estimator for SklearnBox {
    fn test(&self, task: Task, dataset: &Dataset) -> HashMap<String, f32> {
        let x_test = dataset.x_test();
        let y_test = dataset.y_test();
        let y_hat = sklearn_test(&self, x_test, dataset.num_features);

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
