//! Builders for parameters that control various aspects of training.
//!
//! Configuration is based on the documented
//! [XGBoost Parameters](https://xgboost.readthedocs.io/en/latest/parameter.html), see those for
//! more details.
//!
//! Parameters are generally created through builders that provide sensible defaults, and ensure that
//! any given settings are valid when built.
use std::default::Default;
use std::fmt::{self, Display};

pub mod tree;
pub mod learning;
pub mod linear;
pub mod dart;
mod booster;

use super::DMatrix;
pub use self::booster::BoosterType;
use super::booster::CustomObjective;

/// Parameters for training boosters.
/// Created using [`BoosterParametersBuilder`](struct.BoosterParametersBuilder.html).
#[derive(Builder, Clone)]
#[builder(default)]
pub struct BoosterParameters {
    /// Type of booster (tree, linear or DART) along with its parameters.
    ///
    /// *default*: [`GbTree`](enum.BoosterType.html#variant.GbTree)
    booster_type: booster::BoosterType,

    /// Configuration for the learning objective.
    pub(crate) learning_params: learning::LearningTaskParameters,

    /// Whether to print XGBoost's C library's messages or not.
    ///
    /// *default*: `false`
    verbose: bool,

    /// Number of parallel threads XGboost will use (if compiled with multiprocessing support).
    ///
    /// *default*: `None` (XGBoost will automatically determing max threads to use)
    threads: Option<u32>,
}

impl Default for BoosterParameters {
    fn default() -> Self {
        BoosterParameters {
            booster_type: booster::BoosterType::default(),
            learning_params: learning::LearningTaskParameters::default(),
            verbose: false,
            threads: None,
        }
    }
}

impl BoosterParameters {
    /// Get type of booster (tree, linear or DART) along with its parameters.
    pub fn booster_type(&self) -> &booster::BoosterType {
        &self.booster_type
    }

    /// Set type of booster (tree, linear or DART) along with its parameters.
    pub fn set_booster_type<T: Into<booster::BoosterType>>(&mut self, booster_type: T) {
        self.booster_type = booster_type.into();
    }

    /// Get configuration for the learning objective.
    pub fn learning_params(&self) -> &learning::LearningTaskParameters {
        &self.learning_params
    }

    /// Set configuration for the learning objective.
    pub fn set_learning_params<T: Into<learning::LearningTaskParameters>>(&mut self, learning_params: T) {
        self.learning_params = learning_params.into();
    }

    /// Check whether verbose output is enabled or not.
    pub fn verbose(&self) -> bool {
        self.verbose
    }

    /// Set to `true` to enable verbose output from XGBoost's C library.
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Get number of parallel threads XGboost will use (if compiled with multiprocessing support).
    ///
    /// If `None`, XGBoost will determine the number of threads to use automatically.
    pub fn threads(&self) -> &Option<u32> {
        &self.threads
    }

    /// Set number of parallel threads XGBoost will use (if compiled with multiprocessing support).
    ///
    /// If `None`, XGBoost will determine the number of threads to use automatically.
    pub fn set_threads<T: Into<Option<u32>>>(&mut self, threads: T) {
        self.threads = threads.into();
    }

    pub(crate) fn as_string_pairs(&self) -> Vec<(String, String)> {
        let mut v = Vec::new();

        v.extend(self.booster_type.as_string_pairs());
        v.extend(self.learning_params.as_string_pairs());

        v.push(("silent".to_owned(), (!self.verbose as u8).to_string()));

        if let Some(nthread) = self.threads {
            v.push(("nthread".to_owned(), nthread.to_string()));
        }

        v
    }
}

type CustomEvaluation = fn(&[f32], &DMatrix) -> f32;

/// Parameters used by the [`Booster::train`](../struct.Booster.html#method.train) method for training new models.
/// Created using [`TrainingParametersBuilder`](struct.TrainingParametersBuilder.html).
#[derive(Builder, Clone)]
pub struct TrainingParameters<'a> {
    /// Matrix used for training model.
    pub(crate) dtrain: &'a DMatrix,

    /// Number of boosting rounds to use during training.
    ///
    /// *default*: `10`
    #[builder(default="10")]
    pub(crate) boost_rounds: u32,

    /// Configuration for the booster model that will be trained.
    ///
    /// *default*: `BoosterParameters::default()`
    #[builder(default="BoosterParameters::default()")]
    pub(crate) booster_params: BoosterParameters,

    #[builder(default="None")]
    /// Optional list of DMatrix to evaluate against after each boosting round.
    ///
    /// Supplied as a list of tuples of (DMatrix, description). The description is used to differentiate between
    /// different evaluation datasets when output during training.
    ///
    /// *default*: `None`
    pub(crate) evaluation_sets: Option<&'a[(&'a DMatrix, &'a str)]>,

    /// Optional custom objective function to use for training.
    ///
    /// *default*: `None`
    #[builder(default="None")]
    pub(crate) custom_objective_fn: Option<CustomObjective>,

    /// Optional custom evaluation function to use during training.
    ///
    /// *default*: `None`
    #[builder(default="None")]
    pub(crate) custom_evaluation_fn: Option<CustomEvaluation>,
    // TODO: callbacks
}

impl <'a> TrainingParameters<'a> {
    pub fn dtrain(&self) -> &'a DMatrix {
        &self.dtrain
    }

    pub fn set_dtrain(&mut self, dtrain: &'a DMatrix) {
        self.dtrain = dtrain;
    }

    pub fn boost_rounds(&self) -> u32 {
        self.boost_rounds
    }

    pub fn set_boost_rounds(&mut self, boost_rounds: u32) {
        self.boost_rounds = boost_rounds;
    }

    pub fn booster_params(&self) -> &BoosterParameters {
        &self.booster_params
    }

    pub fn set_booster_params<T: Into<BoosterParameters>>(&mut self, booster_params: T) {
        self.booster_params = booster_params.into();
    }

    pub fn evaluation_sets(&self) -> &Option<&'a[(&'a DMatrix, &'a str)]> {
        &self.evaluation_sets
    }

    pub fn set_evaluation_sets(&mut self, evaluation_sets: Option<&'a[(&'a DMatrix, &'a str)]>) {
        self.evaluation_sets = evaluation_sets;
    }

    pub fn custom_objective_fn(&self) -> &Option<CustomObjective> {
        &self.custom_objective_fn
    }

    pub fn set_custom_objective_fn(&mut self, custom_objective_fn: Option<CustomObjective>) {
        self.custom_objective_fn = custom_objective_fn;
    }

    pub fn custom_evaluation_fn(&self) -> &Option<CustomEvaluation> {
        &self.custom_evaluation_fn
    }

    pub fn set_custom_evaluation_fn(&mut self, custom_evaluation_fn: Option<CustomEvaluation>) {
        self.custom_evaluation_fn = custom_evaluation_fn;
    }
}

enum Inclusion {
    Open,
    Closed,
}

struct Interval<T> {
    min: T,
    min_inclusion: Inclusion,
    max: T,
    max_inclusion: Inclusion,
}

impl<T: Display> Display for Interval<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let lower = match self.min_inclusion {
            Inclusion::Closed => '[',
            Inclusion::Open   => '(',
        };
        let upper = match self.max_inclusion {
            Inclusion::Closed => ']',
            Inclusion::Open   => ')',
        };
        write!(f, "{}{}, {}{}", lower, self.min, self.max, upper)
    }
}

impl<T: PartialOrd + Display> Interval<T> {
    fn new(min: T, min_inclusion: Inclusion, max: T, max_inclusion: Inclusion) -> Self {
        Interval { min, min_inclusion, max, max_inclusion }
    }

    fn new_open_open(min: T, max: T) -> Self {
        Interval::new(min, Inclusion::Open, max, Inclusion::Open)
    }

    fn new_open_closed(min: T, max: T) -> Self {
        Interval::new(min, Inclusion::Open, max, Inclusion::Closed)
    }

    fn new_closed_closed(min: T, max: T) -> Self {
        Interval::new(min, Inclusion::Closed, max, Inclusion::Closed)
    }

    fn contains(&self, val: &T) -> bool {
        match self.min_inclusion {
            Inclusion::Closed => if !(val >= &self.min) { return false; },
            Inclusion::Open => if !(val > &self.min) { return false; },
        }
        match self.max_inclusion {
            Inclusion::Closed => if !(val <= &self.max) { return false; },
            Inclusion::Open => if !(val < &self.max) { return false; },
        }
        true
    }

    fn validate(&self, val: &Option<T>, name: &str) -> Result<(), String> {
        match val {
            Some(ref val) => {
                if self.contains(&val) {
                    Ok(())
                } else {
                    Err(format!("Invalid value for '{}' parameter, {} is not in range {}.", name, &val, self))
                }
            },
            None => Ok(())
        }
    }
}
