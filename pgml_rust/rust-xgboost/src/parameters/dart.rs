//! BoosterParameters for controlling
//! [DART](https://xgboost.readthedocs.io/en/latest/tutorials/dart.html) boosters.

use std::default::Default;

use super::Interval;

/// Type of sampling algorithm.
#[derive(Clone)]
pub enum SampleType {
    /// Dropped trees are selected uniformly.
    Uniform,

    /// Dropped trees are selected in proportion to weight.
    Weighted,
}

impl ToString for SampleType {
    fn to_string(&self) -> String {
        match *self {
            SampleType::Uniform => "uniform".to_owned(),
            SampleType::Weighted => "weighted".to_owned(),
        }
    }
}

impl Default for SampleType {
    fn default() -> Self { SampleType::Uniform }
}

/// Type of normalization algorithm.
#[derive(Clone)]
pub enum NormalizeType {
    /// New trees have the same weight of each of dropped trees.
    /// * weight of new trees are 1 / (k + learning_rate)
    /// dropped trees are scaled by a factor of k / (k + learning_rate)
    Tree,

    /// New trees have the same weight of sum of dropped trees (forest).
    ///
    /// * weight of new trees are 1 / (1 + learning_rate)
    /// * droppped trees are scaled by a factor of 1 / (1 + learning_rate)
    Forest,
}

impl ToString for NormalizeType {
    fn to_string(&self) -> String {
        match *self {
            NormalizeType::Tree => "tree".to_owned(),
            NormalizeType::Forest => "forest".to_owned(),
        }
    }
}

impl Default for NormalizeType {
    fn default() -> Self { NormalizeType::Tree }
}

/// Additional parameters for Dart Booster.
#[derive(Builder, Clone)]
#[builder(build_fn(validate = "Self::validate"))]
#[builder(default)]
pub struct DartBoosterParameters {
    /// Type of sampling algorithm.
    sample_type: SampleType,

    /// Type of normalization algorithm.
    normalize_type: NormalizeType,

    /// Dropout rate (a fraction of previous trees to drop during the dropout).
    /// * range: [0.0, 1.0]
    rate_drop: f32,

    /// When this flag is enabled, at least one tree is always dropped during the dropout
    /// (allows Binomial-plus-one or epsilon-dropout from the original DART paper).
    one_drop: bool,

    /// Probability of skipping the dropout procedure during a boosting iteration.
    /// If a dropout is skipped, new trees are added in the same manner as gbtree.
    /// Note that non-zero skip_drop has higher priority than rate_drop or one_drop.
    /// * range: [0.0, 1.0]
    skip_drop: f32,
}

impl Default for DartBoosterParameters {
    fn default() -> Self {
        DartBoosterParameters {
            sample_type: SampleType::default(),
            normalize_type: NormalizeType::default(),
            rate_drop: 0.0,
            one_drop: false,
            skip_drop: 0.0,
        }
    }
}

impl DartBoosterParameters {
    pub(crate) fn as_string_pairs(&self) -> Vec<(String, String)> {
        let mut v = Vec::new();

        v.push(("booster".to_owned(), "dart".to_owned()));

        v.push(("sample_type".to_owned(), self.sample_type.to_string()));
        v.push(("normalize_type".to_owned(), self.normalize_type.to_string()));
        v.push(("rate_drop".to_owned(), self.rate_drop.to_string()));
        v.push(("one_drop".to_owned(), (self.one_drop as u8).to_string()));
        v.push(("skip_drop".to_owned(), self.skip_drop.to_string()));

        v
    }
}

impl DartBoosterParametersBuilder {
    fn validate(&self) -> Result<(), String> {
        Interval::new_closed_closed(0.0, 1.0).validate(&self.rate_drop, "rate_drop")?;
        Interval::new_closed_closed(0.0, 1.0).validate(&self.skip_drop, "skip_drop")?;
        Ok(())
    }
}
