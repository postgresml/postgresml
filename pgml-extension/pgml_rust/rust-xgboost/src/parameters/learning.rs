//! BoosterParameters for configuring learning objectives and evaluation metrics for all
//! booster types.

use std;
use std::default::Default;

use super::Interval;

/// Learning objective used when training a booster model.
pub enum Objective {
    /// Linear regression.
    RegLinear,

    /// Logistic regression.
    RegLogistic,

    /// Logistic regression for binary classification, outputs probability.
    BinaryLogistic,

    /// Logistic regression for binary classification, outputs scores before logistic transformation.
    BinaryLogisticRaw,

    /// GPU version of [`RegLinear`](#variant.RegLinear).
    GpuRegLinear,

    /// GPU version of [`RegLogistic`](#variant.RegLogistic).
    GpuRegLogistic,

    /// GPU version of [`RegBinaryLogistic`](#variant.RegBinaryLogistic).
    GpuBinaryLogistic,

    /// GPU version of [`RegBinaryLogisticRaw`](#variant.RegBinaryLogisticRaw).
    GpuBinaryLogisticRaw,

    /// Poisson regression for count data, outputs mean of poisson distribution.
    CountPoisson,

    /// Cox regression for right censored survival time data (negative values are considered right
    /// censored).
    ///
    /// predictions are returned on the hazard ratio scale (i.e., as `HR = exp(marginal_prediction)`
    /// in the proportional hazard function `h(t) = h0(t) * HR`).
    SurvivalCox,

    /// Multiclass classification using the softmax objective, with given number of classes.
    MultiSoftmax(u32),

    /// Multiclass classification using the softmax objective, with given number of classes.
    ///
    /// Outputs probabilities per class.
    MultiSoftprob(u32),

    /// Ranking task which minimises pairwise loss.
    RankPairwise,

    /// Gamma regression with log-link. Output is the mean of the gamma distribution.
    RegGamma,

    /// Tweedie regression with log-link. Takes an optional **tweedie variance power** parameter
    /// which controls the variance of the Tweedie distribution.
    ///
    /// * Set closer to 2 to shift towards a gamma distribution
    /// * Set closer to 1 to shift towards a Poisson distribution
    ///
    /// *range*: (1, 2)
    ///
    /// Set to `None` to use XGBoost's default (currently `1.5`).
    RegTweedie(Option<f32>),
}

impl Copy for Objective {}

impl Clone for Objective {
    fn clone(&self) -> Self { *self }
}

impl ToString for Objective {
    fn to_string(&self) -> String {
        match *self {
            Objective::RegLinear => "reg:linear".to_owned(),
            Objective::RegLogistic => "reg:logistic".to_owned(),
            Objective::BinaryLogistic => "binary:logistic".to_owned(),
            Objective::BinaryLogisticRaw => "binary:logitraw".to_owned(),
            Objective::GpuRegLinear => "gpu:reg:linear".to_owned(),
            Objective::GpuRegLogistic => "gpu:reg:logistic".to_owned(),
            Objective::GpuBinaryLogistic => "gpu:binary:logistic".to_owned(),
            Objective::GpuBinaryLogisticRaw => "gpu:binary:logitraw".to_owned(),
            Objective::CountPoisson => "count:poisson".to_owned(),
            Objective::SurvivalCox => "survival:cox".to_owned(),
            Objective::MultiSoftmax(_) => "multi:softmax".to_owned(), // num_class conf must also be set
            Objective::MultiSoftprob(_) => "multi:softprob".to_owned(), // num_class conf must also be set
            Objective::RankPairwise => "rank:pairwise".to_owned(),
            Objective::RegGamma => "reg:gamma".to_owned(),
            Objective::RegTweedie(_) => "reg:tweedie".to_owned(),
        }
    }
}

impl Default for Objective {
    fn default() -> Self { Objective::RegLinear }
}

/// Type of evaluation metrics to use during learning.
#[derive(Clone)]
pub enum Metrics {
    /// Automatically selects metrics based on learning objective.
    Auto,

    /// Use custom list of metrics.
    Custom(Vec<EvaluationMetric>),
}

/// Type of evaluation metric used on validation data.
#[derive(Clone)]
pub enum EvaluationMetric {
    /// Root Mean Square Error.
    RMSE,

    /// Mean Absolute Error.
    MAE,

    /// Negative log-likelihood.
    LogLoss,

    // TODO: use error as field if set to 0.5
    /// Binary classification error rate. It is calculated as #(wrong cases)/#(all cases).
    /// For the predictions, the evaluation will regard the instances with prediction value larger than
    /// given threshold as positive instances, and the others as negative instances.
    BinaryErrorRate(f32),

    /// Multiclass classification error rate. It is calculated as #(wrong cases)/#(all cases).
    MultiClassErrorRate,

    /// Multiclass logloss.
    MultiClassLogLoss,

    /// Area under the curve for ranking evaluation.
    AUC,

    /// Normalized Discounted Cumulative Gain.
    NDCG,

    /// NDCG with top N positions cut off.
    NDCGCut(u32),

    /// NDCG with scores of lists without any positive samples evaluated as 0 instead of 1.
    NDCGNegative,

    /// NDCG with scores of lists without any positive samples evaluated as 0 instead of 1, and top
    /// N positions cut off.
    NDCGCutNegative(u32),

    /// Mean average precision.
    MAP,

    /// MAP with top N positions cut off.
    MAPCut(u32),

    /// MAP with scores of lists without any positive samples evaluated as 0 instead of 1.
    MAPNegative,

    /// MAP with scores of lists without any positive samples evaluated as 0 instead of 1, and top
    /// N positions cut off.
    MAPCutNegative(u32),

    /// Negative log likelihood for Poisson regression.
    PoissonLogLoss,

    /// Negative log likelihood for Gamma regression.
    GammaLogLoss,

    /// Negative log likelihood for Cox proportional hazards regression.
    CoxLogLoss,

    /// Residual deviance for Gamma regression.
    GammaDeviance,

    /// Negative log likelihood for Tweedie regression (at a specified value of the tweedie_variance_power parameter).
    TweedieLogLoss,
}

impl ToString for EvaluationMetric {
    fn to_string(&self) -> String {
        match *self {
            EvaluationMetric::RMSE => "rmse".to_owned(),
            EvaluationMetric::MAE => "mae".to_owned(),
            EvaluationMetric::LogLoss => "logloss".to_owned(),
            EvaluationMetric::BinaryErrorRate(t) => {
                if (t - 0.5).abs() < std::f32::EPSILON {
                    "error".to_owned()
                } else {
                    format!("error@{}", t)
                }
            },
            EvaluationMetric::MultiClassErrorRate => "merror".to_owned(),
            EvaluationMetric::MultiClassLogLoss   => "mlogloss".to_owned(),
            EvaluationMetric::AUC                 => "auc".to_owned(),
            EvaluationMetric::NDCG                => "ndcg".to_owned(),
            EvaluationMetric::NDCGCut(n)          => format!("ndcg@{}", n),
            EvaluationMetric::NDCGNegative        => "ndcg-".to_owned(),
            EvaluationMetric::NDCGCutNegative(n)  => format!("ndcg@{}-", n),
            EvaluationMetric::MAP                 => "map".to_owned(),
            EvaluationMetric::MAPCut(n)           => format!("map@{}", n),
            EvaluationMetric::MAPNegative         => "map-".to_owned(),
            EvaluationMetric::MAPCutNegative(n)   => format!("map@{}-", n),
            EvaluationMetric::PoissonLogLoss      => "poisson-nloglik".to_owned(),
            EvaluationMetric::GammaLogLoss        => "gamma-nloglik".to_owned(),
            EvaluationMetric::CoxLogLoss          => "cox-nloglik".to_owned(),
            EvaluationMetric::GammaDeviance       => "gamma-deviance".to_owned(),
            EvaluationMetric::TweedieLogLoss      => "tweedie-nloglik".to_owned(),
        }
    }
}

/// BoosterParameters that configure the learning objective.
///
/// See [`LearningTaskParametersBuilder`](struct.LearningTaskParametersBuilder.html), for details
/// on parameters.
#[derive(Builder, Clone)]
#[builder(build_fn(validate = "Self::validate"))]
#[builder(default)]
pub struct LearningTaskParameters {
    /// Learning objective used when training.
    ///
    /// *default*: [`RegLinear`](enum.Objective.html#variant.RegLinear)
    pub(crate) objective: Objective,

    /// Initial prediction score, i.e. global bias.
    ///
    /// *default*: 0.5
    base_score: f32,

    /// Metrics to use on evaluation data sets during training.
    ///
    /// *default*: [`Auto`](enum.Metrics.html#variant.Auto) (i.e. metrics selected automatically based on objective)
    pub(crate) eval_metrics: Metrics,

    /// Random seed.
    ///
    /// *default*: 0
    seed: u64,
}

impl Default for LearningTaskParameters {
    fn default() -> Self {
        LearningTaskParameters {
            objective: Objective::default(),
            base_score: 0.5,
            eval_metrics: Metrics::Auto,
            seed: 0,
        }
    }
}

impl LearningTaskParameters {
    pub fn objective(&self) -> &Objective {
        &self.objective
    }

    pub fn set_objective<T: Into<Objective>>(&mut self, objective: T) {
        self.objective = objective.into();
    }

    pub fn base_score(&self) -> f32 {
        self.base_score
    }

    pub fn set_base_score(&mut self, base_score: f32) {
        self.base_score = base_score;
    }

    pub fn eval_metrics(&self) -> &Metrics {
        &self.eval_metrics
    }

    pub fn set_eval_metrics<T: Into<Metrics>>(&mut self, eval_metrics: T) {
        self.eval_metrics = eval_metrics.into();
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.seed = seed;
    }

    pub(crate) fn as_string_pairs(&self) -> Vec<(String, String)> {
        let mut v = Vec::new();

        if let Objective::MultiSoftmax(n) = self.objective {
            v.push(("num_class".to_owned(), n.to_string()));
        } else if let Objective::MultiSoftprob(n) = self.objective {
            v.push(("num_class".to_owned(), n.to_string()));
        } else if let Objective::RegTweedie(Some(n)) = self.objective {
            v.push(("tweedie_variance_power".to_owned(), n.to_string()));
        }

        v.push(("objective".to_owned(), self.objective.to_string()));
        v.push(("base_score".to_owned(), self.base_score.to_string()));
        v.push(("seed".to_owned(), self.seed.to_string()));

        if let Metrics::Custom(eval_metrics) = &self.eval_metrics {
            for metric in eval_metrics {
                v.push(("eval_metric".to_owned(), metric.to_string()));
            }
        }

        v
    }
}

impl LearningTaskParametersBuilder {
    fn validate(&self) -> Result<(), String> {
        if let Some(Objective::RegTweedie(variance_power)) = self.objective {
            Interval::new_closed_closed(1.0, 2.0).validate(&variance_power, "tweedie_variance_power")?;
        }
        Ok(())
    }
}
