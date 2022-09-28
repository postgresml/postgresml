use ndarray::Array1;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
/// XGBoost implementation.
///
/// XGBoost is a family of gradient-boosted decision tree algorithms,
/// that are very effective on real-world datasets.
///
/// It uses its own dense matrix.
use xgboost::{parameters, Booster, DMatrix};

use crate::orm::dataset::Dataset;
use crate::orm::estimator::calc_metrics;
use crate::orm::estimator::BoosterBox;
use crate::orm::search::Search;
use crate::orm::task::Task;
use crate::orm::Hyperparams;
// use crate::orm::estimator::Estimator;

use pgx::*;
use serde_json;

#[pg_extern]
fn xgboost_version() -> String {
    String::from("1.62")
}

/// Train an XGBoost estimator.
pub fn xgboost_train(
    task: Task,
    dataset: &Dataset,
    hyperparams: &serde_json::Map<std::string::String, serde_json::Value>,
) -> BoosterBox {
    let mut dtrain = DMatrix::from_dense(dataset.x_train(), dataset.num_train_rows).unwrap();
    let mut dtest = DMatrix::from_dense(dataset.x_test(), dataset.num_test_rows).unwrap();
    dtrain.set_labels(dataset.y_train()).unwrap();
    dtest.set_labels(dataset.y_test()).unwrap();

    // specify datasets to evaluate against during training
    let evaluation_sets = &[(&dtrain, "train"), (&dtest, "test")];

    // configure objectives, metrics, etc.
    let learning_params = parameters::learning::LearningTaskParametersBuilder::default()
        .objective(match task {
            Task::regression => xgboost::parameters::learning::Objective::RegLinear,
            Task::classification => {
                xgboost::parameters::learning::Objective::MultiSoftmax(dataset.distinct_labels())
                // [0, num_class)
            }
        })
        .build()
        .unwrap();

    // configure the tree-based learning model's parameters
    let tree_params = parameters::tree::TreeBoosterParametersBuilder::default()
        .max_depth(match hyperparams.get("max_depth") {
            Some(value) => value.as_u64().unwrap_or(2) as u32,
            None => 2,
        })
        .eta(match hyperparams.get("eta") {
            Some(value) => value.as_f64().unwrap_or(0.3) as f32,
            None => match hyperparams.get("learning_rate") {
                Some(value) => value.as_f64().unwrap_or(0.3) as f32,
                None => 0.3,
            },
        })
        .gamma(match hyperparams.get("gamma") {
            Some(value) => value.as_f64().unwrap_or(0.0) as f32,
            None => match hyperparams.get("min_split_loss") {
                Some(value) => value.as_f64().unwrap_or(0.0) as f32,
                None => 0.0,
            },
        })
        .min_child_weight(match hyperparams.get("min_child_weight") {
            Some(value) => value.as_f64().unwrap_or(1.0) as f32,
            None => 1.0,
        })
        .max_delta_step(match hyperparams.get("max_delta_step") {
            Some(value) => value.as_f64().unwrap_or(0.0) as f32,
            None => 0.0,
        })
        .subsample(match hyperparams.get("subsample") {
            Some(value) => value.as_f64().unwrap_or(1.0) as f32,
            None => 1.0,
        })
        .lambda(match hyperparams.get("lambda") {
            Some(value) => value.as_f64().unwrap_or(1.0) as f32,
            None => 1.0,
        })
        .alpha(match hyperparams.get("alpha") {
            Some(value) => value.as_f64().unwrap_or(0.0) as f32,
            None => 0.0,
        })
        .tree_method(match hyperparams.get("tree_method") {
            Some(value) => match value.as_str().unwrap_or("auto") {
                "auto" => parameters::tree::TreeMethod::Auto,
                "exact" => parameters::tree::TreeMethod::Exact,
                "approx" => parameters::tree::TreeMethod::Approx,
                "hist" => parameters::tree::TreeMethod::Hist,
                _ => parameters::tree::TreeMethod::Auto,
            },

            None => parameters::tree::TreeMethod::Auto,
        })
        .sketch_eps(match hyperparams.get("sketch_eps") {
            Some(value) => value.as_f64().unwrap_or(0.03) as f32,
            None => 0.03,
        })
        .max_leaves(match hyperparams.get("max_leaves") {
            Some(value) => value.as_u64().unwrap_or(0) as u32,
            None => 0,
        })
        .max_bin(match hyperparams.get("max_bin") {
            Some(value) => value.as_u64().unwrap_or(256) as u32,
            None => 256,
        })
        .num_parallel_tree(match hyperparams.get("num_parallel_tree") {
            Some(value) => value.as_u64().unwrap_or(1) as u32,
            None => 1,
        })
        .grow_policy(match hyperparams.get("grow_policy") {
            Some(value) => match value.as_str().unwrap_or("depthwise") {
                "depthwise" => parameters::tree::GrowPolicy::Depthwise,
                "lossguide" => parameters::tree::GrowPolicy::LossGuide,
                _ => parameters::tree::GrowPolicy::Depthwise,
            },

            None => parameters::tree::GrowPolicy::Depthwise,
        })
        .build()
        .unwrap();

    let linear_params = parameters::linear::LinearBoosterParametersBuilder::default()
        .alpha(match hyperparams.get("alpha") {
            Some(value) => value.as_f64().unwrap_or(0.0) as f32,
            None => 0.0,
        })
        .lambda(match hyperparams.get("lambda") {
            Some(value) => value.as_f64().unwrap_or(0.0) as f32,
            None => 0.0,
        })
        .build()
        .unwrap();

    let dart_params = parameters::dart::DartBoosterParametersBuilder::default()
        .rate_drop(match hyperparams.get("rate_drop") {
            Some(value) => value.as_f64().unwrap_or(0.0) as f32,
            None => 0.0,
        })
        .one_drop(match hyperparams.get("one_drop") {
            Some(value) => value.as_u64().unwrap_or(0) != 0,
            None => false,
        })
        .skip_drop(match hyperparams.get("skip_drop") {
            Some(value) => value.as_f64().unwrap_or(0.0) as f32,
            None => 0.0,
        })
        .sample_type(match hyperparams.get("sample_type") {
            Some(value) => match value.as_str().unwrap_or("uniform") {
                "uniform" => parameters::dart::SampleType::Uniform,
                "weighted" => parameters::dart::SampleType::Weighted,
                _ => parameters::dart::SampleType::Uniform,
            },
            None => parameters::dart::SampleType::Uniform,
        })
        .normalize_type(match hyperparams.get("normalize_type") {
            Some(value) => match value.as_str().unwrap_or("tree") {
                "tree" => parameters::dart::NormalizeType::Tree,
                "forest" => parameters::dart::NormalizeType::Forest,
                _ => parameters::dart::NormalizeType::Tree,
            },
            None => parameters::dart::NormalizeType::Tree,
        })
        .build()
        .unwrap();

    // overall configuration for Booster
    let booster_params = parameters::BoosterParametersBuilder::default()
        .booster_type(match hyperparams.get("booster") {
            Some(value) => match value.as_str().unwrap_or("gbtree") {
                "gbtree" => parameters::BoosterType::Tree(tree_params),
                "linear" => parameters::BoosterType::Linear(linear_params),
                "dart" => parameters::BoosterType::Dart(dart_params),
                _ => parameters::BoosterType::Tree(tree_params),
            },
            None => parameters::BoosterType::Tree(tree_params),
        })
        .learning_params(learning_params)
        .verbose(true)
        .build()
        .unwrap();

    // overall configuration for training/evaluation
    let params = parameters::TrainingParametersBuilder::default()
        .dtrain(&dtrain) // dataset to train with
        .boost_rounds(match hyperparams.get("n_estimators") {
            Some(value) => value.as_u64().unwrap_or(10) as u32,
            None => 10,
        }) // number of training iterations
        .booster_params(booster_params) // model parameters
        .evaluation_sets(Some(evaluation_sets)) // optional datasets to evaluate against in each iteration
        .build()
        .unwrap();

    // train model, and print evaluation data
    let bst = match Booster::train(&params) {
        Ok(bst) => bst,
        Err(err) => error!("{}", err),
    };

    BoosterBox::new(bst)
}

/// Serialize an XGBoost estimator into bytes.
pub fn xgboost_save(estimator: &BoosterBox) -> Vec<u8> {
    let r: u64 = rand::random();
    let path = format!("/tmp/pgml_{}.bin", r);

    estimator.save(std::path::Path::new(&path)).unwrap();

    let bytes = std::fs::read(&path).unwrap();

    std::fs::remove_file(&path).unwrap();

    bytes
}

/// Load an XGBoost estimator from bytes.
pub fn xgboost_load(data: &[u8]) -> BoosterBox {
    let bst = Booster::load_buffer(data).unwrap();
    BoosterBox::new(bst)
}

/// Validate a trained estimator against the test dataset.
pub fn xgboost_test(estimator: &BoosterBox, dataset: &Dataset) -> Vec<f32> {
    let mut x_test = DMatrix::from_dense(dataset.x_test(), dataset.num_test_rows).unwrap();
    x_test.set_labels(dataset.y_test()).unwrap();

    estimator.predict(&x_test).unwrap()
}

/// Predict a novel datapoint using the XGBoost estimator.
pub fn xgboost_predict(estimator: &BoosterBox, x: &[f32]) -> f32 {
    let x = DMatrix::from_dense(x, 1).unwrap();
    estimator.predict(&x).unwrap()[0]
}

pub fn xgboost_search(
    task: Task,
    search: Search, // TODO: support random, only grid supported at the moment
    dataset: &Dataset,
    hyperparams: &Hyperparams,
    search_params: &Hyperparams,
) -> (BoosterBox, Hyperparams) {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bindings/wrappers.py"
    ));

    // Get the cartesian product of hyperparams.
    // Couldn't figure out an obvious to do this in Rust :/
    let params = Python::with_gil(|py| -> String {
        let module = PyModule::from_code(py, module, "", "").unwrap();
        let search = module.getattr("generate_params").unwrap();
        let params: String = search
            .call1(PyTuple::new(
                py,
                &[serde_json::to_string(search_params).unwrap()],
            ))
            .unwrap()
            .extract()
            .unwrap();
        params
    });

    let combinations: Vec<Hyperparams> = serde_json::from_str(&params).unwrap();

    let mut best_metric = 0.0;
    let mut best_params = Hyperparams::new();
    let mut best_bst: Option<BoosterBox> = None;

    for params in &combinations {
        // Merge hyperparams with candidate hyperparameters.
        let mut params = params.clone();
        params.extend(hyperparams.clone());

        // Train
        let bst = xgboost_train(task, dataset, &params);

        // Test
        let y_hat =
            Array1::from_shape_vec(dataset.num_test_rows, xgboost_test(&bst, dataset)).unwrap();
        let y_test =
            Array1::from_shape_vec(dataset.num_test_rows, dataset.y_test().to_vec()).unwrap();

        let metrics = calc_metrics(&y_test, &y_hat, dataset.distinct_labels(), task);

        // Compare
        let metric = match task {
            Task::regression => metrics.get("r2").unwrap(),

            Task::classification => metrics.get("f1").unwrap(),
        };

        if metric > &best_metric {
            best_metric = *metric;
            best_params = params.clone();
            best_bst = Some(bst);
        }
    }

    // Return the best
    (best_bst.unwrap(), best_params)
}
