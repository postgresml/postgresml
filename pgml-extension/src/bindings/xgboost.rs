/// XGBoost implementation.
///
/// XGBoost is a family of gradient-boosted decision tree algorithms,
/// that are very effective on real-world datasets.
///
/// It uses its own dense matrix.
use anyhow::{anyhow, Result};
use xgboost::parameters::tree::*;
use xgboost::parameters::*;
use xgboost::{Booster, DMatrix};

use crate::orm::dataset::Dataset;
use crate::orm::Hyperparams;

use crate::bindings::Bindings;

use pgrx::*;

#[pg_extern]
fn xgboost_version() -> String {
    String::from("1.62")
}

fn get_dart_params(hyperparams: &Hyperparams) -> dart::DartBoosterParameters {
    let mut params = dart::DartBoosterParametersBuilder::default();
    for (key, value) in hyperparams {
        match key.as_str() {
            "rate_drop" => params.rate_drop(value.as_f64().unwrap() as f32),
            "one_drop" => params.one_drop(value.as_bool().unwrap()),
            "skip_drop" => params.skip_drop(value.as_f64().unwrap() as f32),
            "sample_type" => match value.as_str().unwrap() {
                "uniform" => params.sample_type(dart::SampleType::Uniform),
                "weighted" => params.sample_type(dart::SampleType::Weighted),
                _ => panic!("Unknown {:?}: {:?}", key, value),
            },
            "normalize_type" => match value.as_str().unwrap() {
                "tree" => params.normalize_type(dart::NormalizeType::Tree),
                "forest" => params.normalize_type(dart::NormalizeType::Forest),
                _ => panic!("Unknown {:?}: {:?}", key, value),
            },
            "booster" | "n_estimators" | "boost_rounds" => &mut params, // Valid but not relevant to this section
            "nthread" => &mut params,
            _ => panic!("Unknown {:?}: {:?}", key, value),
        };
    }
    params.build().unwrap()
}

fn get_linear_params(hyperparams: &Hyperparams) -> linear::LinearBoosterParameters {
    let mut params = linear::LinearBoosterParametersBuilder::default();
    for (key, value) in hyperparams {
        match key.as_str() {
            "alpha" | "reg_alpha" => params.alpha(value.as_f64().unwrap() as f32),
            "lambda" | "reg_lambda" => params.lambda(value.as_f64().unwrap() as f32),
            "updater" => match value.as_str().unwrap() {
                "shotgun" => params.updater(linear::LinearUpdate::Shotgun),
                "coord_descent" => params.updater(linear::LinearUpdate::CoordDescent),
                _ => panic!("Unknown {:?}: {:?}", key, value),
            },
            "booster" | "n_estimators" | "boost_rounds" => &mut params, // Valid but not relevant to this section
            "nthread" => &mut params,
            _ => panic!("Unknown {:?}: {:?}", key, value),
        };
    }
    params.build().unwrap()
}

fn get_tree_params(hyperparams: &Hyperparams) -> tree::TreeBoosterParameters {
    let mut params = tree::TreeBoosterParametersBuilder::default();
    for (key, value) in hyperparams {
        match key.as_str() {
            "eta" | "learning_rate" => params.eta(value.as_f64().unwrap() as f32),
            "gamma" | "min_split_loss" => params.gamma(value.as_f64().unwrap() as f32),
            "max_depth" => params.max_depth(value.as_u64().unwrap() as u32),
            "min_child_weight" => params.min_child_weight(value.as_f64().unwrap() as f32),
            "max_delta_step" => params.max_delta_step(value.as_f64().unwrap() as f32),
            "subsample" => params.subsample(value.as_f64().unwrap() as f32),
            "colsample_bytree" => params.colsample_bytree(value.as_f64().unwrap() as f32),
            "colsample_bylevel" => params.colsample_bylevel(value.as_f64().unwrap() as f32),
            "lambda" | "reg_lambda" => params.lambda(value.as_f64().unwrap() as f32),
            "alpha" | "reg_alpha" => params.alpha(value.as_f64().unwrap() as f32),
            "tree_method" => match value.as_str().unwrap() {
                "auto" => params.tree_method(TreeMethod::Auto),
                "exact" => params.tree_method(TreeMethod::Exact),
                "approx" => params.tree_method(TreeMethod::Approx),
                "hist" => params.tree_method(TreeMethod::Hist),
                "gpu_exact" => params.tree_method(TreeMethod::GpuExact),
                "gpu_hist" => params.tree_method(TreeMethod::GpuHist),
                _ => panic!("Unknown hyperparameter {:?}: {:?}", key, value),
            },
            "sketch_eps" => params.sketch_eps(value.as_f64().unwrap() as f32),
            "scale_pos_weight" => params.scale_pos_weight(value.as_f64().unwrap() as f32),
            "updater" => match value.as_array() {
                Some(array) => {
                    let mut v = Vec::new();
                    for value in array {
                        match value.as_str().unwrap() {
                            "grow_col_maker" => v.push(TreeUpdater::GrowColMaker),
                            "dist_col" => v.push(TreeUpdater::DistCol),
                            "grow_hist_maker" => v.push(TreeUpdater::GrowHistMaker),
                            "grow_local_hist_maker" => v.push(TreeUpdater::GrowLocalHistMaker),
                            "grow_sk_maker" => v.push(TreeUpdater::GrowSkMaker),
                            "sync" => v.push(TreeUpdater::Sync),
                            "refresh" => v.push(TreeUpdater::Refresh),
                            "prune" => v.push(TreeUpdater::Prune),
                            _ => panic!("Unknown hyperparameter {:?}: {:?}", key, value),
                        }
                    }
                    params.updater(v)
                }
                _ => panic!("updater should be a JSON array. Got: {:?}", value),
            },
            "refresh_leaf" => params.refresh_leaf(value.as_bool().unwrap()),
            "process_type" => match value.as_str().unwrap() {
                "default" => params.process_type(ProcessType::Default),
                "update" => params.process_type(ProcessType::Update),
                _ => panic!("Unknown hyperparameter {:?}: {:?}", key, value),
            },
            "grow_policy" => match value.as_str().unwrap() {
                "depthwise" => params.grow_policy(GrowPolicy::Depthwise),
                "loss_guide" => params.grow_policy(GrowPolicy::LossGuide),
                _ => panic!("Unknown hyperparameter {:?}: {:?}", key, value),
            },
            "predictor" => match value.as_str().unwrap() {
                "cpu" => params.predictor(Predictor::Cpu),
                "gpu" => params.predictor(Predictor::Gpu),
                _ => panic!("Unknown hyperparameter {:?}: {:?}", key, value),
            },
            "max_leaves" => params.max_leaves(value.as_u64().unwrap() as u32),
            "max_bin" => params.max_bin(value.as_u64().unwrap() as u32),
            "booster" | "n_estimators" | "boost_rounds" | "eval_metric" | "objective" => &mut params, // Valid but not relevant to this section
            "nthread" => &mut params,
            "random_state" => &mut params,
            _ => panic!("Unknown hyperparameter {:?}: {:?}", key, value),
        };
    }
    params.build().unwrap()
}

pub fn fit_regression(dataset: &Dataset, hyperparams: &Hyperparams) -> Result<Box<dyn Bindings>> {
    fit(dataset, hyperparams, learning::Objective::RegLinear)
}

pub fn fit_classification(dataset: &Dataset, hyperparams: &Hyperparams) -> Result<Box<dyn Bindings>> {
    fit(
        dataset,
        hyperparams,
        learning::Objective::MultiSoftprob(dataset.num_distinct_labels.try_into().unwrap()),
    )
}

fn eval_metric_from_string(name: &str) -> learning::EvaluationMetric {
    match name {
        "rmse" => learning::EvaluationMetric::RMSE,
        "mae" => learning::EvaluationMetric::MAE,
        "logloss" => learning::EvaluationMetric::LogLoss,
        "merror" => learning::EvaluationMetric::MultiClassErrorRate,
        "mlogloss" => learning::EvaluationMetric::MultiClassLogLoss,
        "auc" => learning::EvaluationMetric::AUC,
        "ndcg" => learning::EvaluationMetric::NDCG,
        "ndcg-" => learning::EvaluationMetric::NDCGNegative,
        "map" => learning::EvaluationMetric::MAP,
        "map-" => learning::EvaluationMetric::MAPNegative,
        "poisson-nloglik" => learning::EvaluationMetric::PoissonLogLoss,
        "gamma-nloglik" => learning::EvaluationMetric::GammaLogLoss,
        "cox-nloglik" => learning::EvaluationMetric::CoxLogLoss,
        "gamma-deviance" => learning::EvaluationMetric::GammaDeviance,
        "tweedie-nloglik" => learning::EvaluationMetric::TweedieLogLoss,
        _ => error!("Unknown eval_metric: {:?}", name),
    }
}

fn objective_from_string(name: &str, dataset: &Dataset) -> learning::Objective {
    match name {
        "reg:linear" => learning::Objective::RegLinear,
        "reg:logistic" => learning::Objective::RegLogistic,
        "binary:logistic" => learning::Objective::BinaryLogistic,
        "binary:logitraw" => learning::Objective::BinaryLogisticRaw,
        "gpu:reg:linear" => learning::Objective::GpuRegLinear,
        "gpu:reg:logistic" => learning::Objective::GpuRegLogistic,
        "gpu:binary:logistic" => learning::Objective::GpuBinaryLogistic,
        "gpu:binary:logitraw" => learning::Objective::GpuBinaryLogisticRaw,
        "count:poisson" => learning::Objective::CountPoisson,
        "survival:cox" => learning::Objective::SurvivalCox,
        "multi:softmax" => learning::Objective::MultiSoftmax(dataset.num_distinct_labels.try_into().unwrap()),
        "multi:softprob" => learning::Objective::MultiSoftprob(dataset.num_distinct_labels.try_into().unwrap()),
        "rank:pairwise" => learning::Objective::RankPairwise,
        "reg:gamma" => learning::Objective::RegGamma,
        "reg:tweedie" => learning::Objective::RegTweedie(Some(dataset.num_distinct_labels as f32)),
        _ => error!("Unknown objective: {:?}", name),
    }
}

fn fit(dataset: &Dataset, hyperparams: &Hyperparams, objective: learning::Objective) -> Result<Box<dyn Bindings>> {
    // split the train/test data into DMatrix
    let mut dtrain = DMatrix::from_dense(&dataset.x_train, dataset.num_train_rows).unwrap();
    let mut dtest = DMatrix::from_dense(&dataset.x_test, dataset.num_test_rows).unwrap();
    dtrain.set_labels(&dataset.y_train).unwrap();
    dtest.set_labels(&dataset.y_test).unwrap();

    // specify datasets to evaluate against during training
    let evaluation_sets = &[(&dtrain, "train"), (&dtest, "test")];

    let seed = match hyperparams.get("random_state") {
        Some(value) => value.as_u64().unwrap(),
        None => 0,
    };
    let eval_metrics = match hyperparams.get("eval_metric") {
        Some(metrics) => {
            if metrics.is_array() {
                learning::Metrics::Custom(
                    metrics
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|metric| eval_metric_from_string(metric.as_str().unwrap()))
                        .collect(),
                )
            } else {
                learning::Metrics::Custom(Vec::from([eval_metric_from_string(metrics.as_str().unwrap())]))
            }
        }
        None => learning::Metrics::Auto,
    };
    let learning_params = match learning::LearningTaskParametersBuilder::default()
        .objective(match hyperparams.get("objective") {
            Some(value) => objective_from_string(value.as_str().unwrap(), dataset),
            None => objective,
        })
        .eval_metrics(eval_metrics)
        .seed(seed)
        .build()
    {
        Ok(params) => params,
        Err(e) => error!("Failed to parse learning params:\n\n{}", e),
    };

    // overall configuration for Booster
    let booster_params = match BoosterParametersBuilder::default()
        .learning_params(learning_params)
        .booster_type(match hyperparams.get("booster") {
            Some(value) => match value.as_str().unwrap() {
                "gbtree" => BoosterType::Tree(get_tree_params(hyperparams)),
                "linear" => BoosterType::Linear(get_linear_params(hyperparams)),
                "dart" => BoosterType::Dart(get_dart_params(hyperparams)),
                _ => panic!("Unknown booster: {:?}", value),
            },
            _ => BoosterType::Tree(get_tree_params(hyperparams)),
        })
        .threads(
            hyperparams
                .get("nthread")
                .map(|value| value.as_i64().expect("nthread must be an integer") as u32),
        )
        .verbose(true)
        .build()
    {
        Ok(params) => params,
        Err(e) => error!("Failed to configure booster:\n\n{}", e),
    };

    let mut builder = TrainingParametersBuilder::default();
    // number of training iterations is aliased
    match hyperparams.get("n_estimators") {
        Some(value) => builder.boost_rounds(value.as_u64().unwrap() as u32),
        None => match hyperparams.get("boost_rounds") {
            Some(value) => builder.boost_rounds(value.as_u64().unwrap() as u32),
            None => &mut builder,
        },
    };

    let params = match builder
        // dataset to train with
        .dtrain(&dtrain)
        // optional datasets to evaluate against in each iteration
        .evaluation_sets(Some(evaluation_sets))
        // model parameters
        .booster_params(booster_params)
        .build()
    {
        Ok(params) => params,
        Err(e) => error!("Failed to create training parameters:\n\n{}", e),
    };

    // train model, and print evaluation data
    let booster = match Booster::train(&params) {
        Ok(booster) => booster,
        Err(e) => error!("Failed to train model:\n\n{}", e),
    };

    let softmax_objective = match hyperparams.get("objective") {
        Some(value) => match value.as_str().unwrap() {
            "multi:softmax" => true,
            _ => false,
        },
        None => false,
    };
    Ok(Box::new(Estimator { softmax_objective, estimator: booster }))
}

pub struct Estimator {
    softmax_objective: bool,
    estimator: xgboost::Booster,
}

unsafe impl Send for Estimator {}
unsafe impl Sync for Estimator {}

impl std::fmt::Debug for Estimator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        formatter.debug_struct("Estimator").finish()
    }
}

impl Bindings for Estimator {
    fn predict(&self, features: &[f32], num_features: usize, num_classes: usize) -> Result<Vec<f32>> {
        let x = DMatrix::from_dense(features, features.len() / num_features)?;
        let y = self.estimator.predict(&x)?;
        if self.softmax_objective {
            return Ok(y);
        }
        Ok(match num_classes {
            0 => y,
            _ => y
                .chunks(num_classes)
                .map(|probabilities| {
                    probabilities
                        .iter()
                        .enumerate()
                        .max_by(|(_, a), (_, b)| a.total_cmp(b))
                        .map(|(index, _)| index)
                        .unwrap() as f32
                })
                .collect::<Vec<f32>>(),
        })
    }

    fn predict_proba(&self, features: &[f32], num_features: usize) -> Result<Vec<f32>> {
        let x = DMatrix::from_dense(features, features.len() / num_features)?;
        Ok(self.estimator.predict(&x)?)
    }

    /// Serialize self to bytes
    fn to_bytes(&self) -> Result<Vec<u8>> {
        let r: u64 = rand::random();
        let path = format!("/tmp/pgml_{}.bin", r);
        self.estimator.save(std::path::Path::new(&path))?;
        let bytes = std::fs::read(&path)?;
        std::fs::remove_file(&path)?;
        Ok(bytes)
    }

    /// Deserialize self from bytes, with additional context
    fn from_bytes(bytes: &[u8], hyperparams: &JsonB) -> Result<Box<dyn Bindings>>
    where
        Self: Sized,
    {
        let mut estimator = Booster::load_buffer(bytes);
        if estimator.is_err() {
            // backward compatibility w/ 2.0.0
            estimator = Booster::load_buffer(&bytes[16..]);
        }

        let mut estimator = estimator?;

        // Get concurrency setting
        let concurrency: i64 = Spi::get_one(
            "
            SELECT COALESCE(
                current_setting('pgml.predict_concurrency', true),
                '2'
            )::bigint",
        )?
        .unwrap();

        estimator
            .set_param("nthread", &concurrency.to_string())
            .map_err(|e| anyhow!("could not set nthread XGBoost parameter: {e}"))?;

        let objective_opt = hyperparams.0.get("objective").and_then(|v| v.as_str());
        let softmax_objective = match objective_opt {
            Some("multi:softmax") => true,
            _ => false,
        };

        Ok(Box::new(Estimator { softmax_objective, estimator }))
    }
}
