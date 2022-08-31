//! BoosterParameters for configuring linear boosters.

use std::default::Default;

/// Linear model algorithm.
#[derive(Clone)]
pub enum LinearUpdate {
    /// Parallel coordinate descent algorithm based on shotgun algorithm. Uses ‘hogwild’ parallelism and
    /// therefore produces a nondeterministic solution on each run.
    Shotgun,

    /// Ordinary coordinate descent algorithm. Also multithreaded but still produces a deterministic solution.
    CoordDescent,
}

impl ToString for LinearUpdate {
    fn to_string(&self) -> String {
        match *self {
            LinearUpdate::Shotgun => "shotgun".to_owned(),
            LinearUpdate::CoordDescent => "coord_descent".to_owned(),
        }
    }
}

impl Default for LinearUpdate {
    fn default() -> Self { LinearUpdate::Shotgun }
}

/// BoosterParameters for Linear Booster.
#[derive(Builder, Clone)]
#[builder(default)]
pub struct LinearBoosterParameters {
    /// L2 regularization term on weights, increase this value will make model more conservative.
    /// Normalised to number of training examples.
    ///
    /// * default: 0.0
    lambda: f32,

    /// L1 egularization term on weights, increase this value will make model more conservative.
    /// Normalised to number of training examples.
    ///
    /// * default: 0.0
    alpha: f32,

    /// Linear model algorithm.
    ///
    /// * default: `LinearUpdate::Shotgun`
    updater: LinearUpdate,
}


impl LinearBoosterParameters {
    pub(crate) fn as_string_pairs(&self) -> Vec<(String, String)> {
        let mut v = Vec::new();

        v.push(("booster".to_owned(), "gblinear".to_owned()));

        v.push(("lambda".to_owned(), self.lambda.to_string()));
        v.push(("alpha".to_owned(), self.alpha.to_string()));
        v.push(("updater".to_owned(), self.updater.to_string()));

        v
    }
}

impl Default for LinearBoosterParameters {
    fn default() -> Self {
        LinearBoosterParameters {
            lambda: 0.0,
            alpha: 0.0,
            updater: LinearUpdate::default(),
        }
    }
}
