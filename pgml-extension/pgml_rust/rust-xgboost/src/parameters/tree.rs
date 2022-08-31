//! BoosterParameters for controlling tree boosters.
//!
//!
use std::default::Default;

use super::Interval;

/// The tree construction algorithm used in XGBoost (see description in the
/// [reference paper](http://arxiv.org/abs/1603.02754)).
///
/// Distributed and external memory version only support approximate algorithm.
#[derive(Clone)]
pub enum TreeMethod {
    /// Use heuristic to choose faster one.
    ///
    /// * For small to medium dataset, exact greedy will be used.
    /// * For very large-dataset, approximate algorithm will be chosen.
    /// * Because old behavior is always use exact greedy in single machine, user will get a message when
    ///   approximate algorithm is chosen to notify this choice.
    Auto,

    /// Exact greedy algorithm.
    Exact,

    /// Approximate greedy algorithm using sketching and histogram.
    Approx,

    /// Fast histogram optimized approximate greedy algorithm. It uses some performance improvements
    /// such as bins caching.
    Hist,

    /// GPU implementation of exact algorithm.
    GpuExact,

    /// GPU implementation of hist algorithm.
    GpuHist,
}

impl ToString for TreeMethod {
    fn to_string(&self) -> String {
        match *self {
            TreeMethod::Auto => "auto".to_owned(),
            TreeMethod::Exact => "exact".to_owned(),
            TreeMethod::Approx => "approx".to_owned(),
            TreeMethod::Hist => "hist".to_owned(),
            TreeMethod::GpuExact => "gpu_exact".to_owned(),
            TreeMethod::GpuHist => "gpu_hist".to_owned(),
        }
    }
}

impl Default for TreeMethod {
    fn default() -> Self { TreeMethod::Auto }
}

impl From<String> for TreeMethod
{
    fn from(s: String) -> Self
    {
      use std::borrow::Borrow;
      Self::from(s.borrow())
    }
}

impl<'a> From<&'a str> for TreeMethod
{
    fn from(s: &'a str) -> Self
    {
      match s
      {
        "auto" => TreeMethod::Auto,
        "exact" => TreeMethod::Exact,
        "approx" => TreeMethod::Approx,
        "hist" => TreeMethod::Hist,
        "gpu_exact" => TreeMethod::GpuExact,
        "gpu_hist" => TreeMethod::GpuHist,
        _ => panic!("no known tree_method for {}", s)
      }
    }
}

/// Provides a modular way to construct and to modify the trees. This is an advanced parameter that is usually set
/// automatically, depending on some other parameters. However, it could be also set explicitly by a user.
#[derive(Clone)]
pub enum TreeUpdater {
    /// Non-distributed column-based construction of trees.
    GrowColMaker,

    /// Distributed tree construction with column-based data splitting mode.
    DistCol,

    /// Distributed tree construction with row-based data splitting based on global proposal of histogram counting.
    GrowHistMaker,

    /// Based on local histogram counting.
    GrowLocalHistMaker,

    /// Uses the approximate sketching algorithm.
    GrowSkMaker,

    /// Synchronizes trees in all distributed nodes.
    Sync,

    /// Refreshes tree’s statistics and/or leaf values based on the current data.
    /// Note that no random subsampling of data rows is performed.
    Refresh,

    /// Prunes the splits where loss < min_split_loss (or gamma).
    Prune,
}

impl ToString for TreeUpdater {
    fn to_string(&self) -> String {
        match *self {
            TreeUpdater::GrowColMaker => "grow_colmaker".to_owned(),
            TreeUpdater::DistCol => "distcol".to_owned(),
            TreeUpdater::GrowHistMaker => "grow_histmaker".to_owned(),
            TreeUpdater::GrowLocalHistMaker => "grow_local_histmaker".to_owned(),
            TreeUpdater::GrowSkMaker => "grow_skmaker".to_owned(),
            TreeUpdater::Sync => "sync".to_owned(),
            TreeUpdater::Refresh => "refresh".to_owned(),
            TreeUpdater::Prune => "prune".to_owned(),
        }
    }
}

/// A type of boosting process to run.
#[derive(Clone)]
pub enum ProcessType {
    /// The normal boosting process which creates new trees.
    Default,

    /// Starts from an existing model and only updates its trees. In each boosting iteration,
    /// a tree from the initial model is taken, a specified sequence of updater plugins is run for that tree,
    /// and a modified tree is added to the new model. The new model would have either the same or smaller number of
    /// trees, depending on the number of boosting iteratons performed.
    /// Currently, the following built-in updater plugins could be meaningfully used with this process type:
    /// 'refresh', 'prune'. With 'update', one cannot use updater plugins that create new trees.
    Update,
}

impl ToString for ProcessType {
    fn to_string(&self) -> String {
        match *self {
            ProcessType::Default => "default".to_owned(),
            ProcessType::Update => "update".to_owned(),
        }
    }
}

impl Default for ProcessType {
    fn default() -> Self { ProcessType::Default }
}

/// Controls the way new nodes are added to the tree.
#[derive(Clone)]
pub enum GrowPolicy {
    /// Split at nodes closest to the root.
    Depthwise,

    /// Split at noeds with highest loss change.
    LossGuide,
}

impl ToString for GrowPolicy {
    fn to_string(&self) -> String {
        match *self {
            GrowPolicy::Depthwise => "depthwise".to_owned(),
            GrowPolicy::LossGuide => "lossguide".to_owned(),
        }
    }
}

impl Default for GrowPolicy {
    fn default() -> Self { GrowPolicy::Depthwise }
}

/// The type of predictor algorithm to use. Provides the same results but allows the use of GPU or CPU.
#[derive(Clone)]
pub enum Predictor {
    /// Multicore CPU prediction algorithm.
    Cpu,

    /// Prediction using GPU. Default for ‘gpu_exact’ and ‘gpu_hist’ tree method.
    Gpu,
}

impl ToString for Predictor {
    fn to_string(&self) -> String {
        match *self {
            Predictor::Cpu => "cpu_predictor".to_owned(),
            Predictor::Gpu => "gpu_predictor".to_owned(),
        }
    }
}

impl Default for Predictor {
    fn default() -> Self { Predictor::Cpu }
}

/// BoosterParameters for Tree Booster. Create using
/// [`TreeBoosterParametersBuilder`](struct.TreeBoosterParametersBuilder.html).
#[derive(Builder, Clone)]
#[builder(build_fn(validate = "Self::validate"))]
#[builder(default)]
pub struct TreeBoosterParameters {
    /// Step size shrinkage used in update to prevents overfitting. After each boosting step, we can directly
    /// get the weights of new features, and eta actually shrinks the feature weights to make the boosting process
    /// more conservative.
    ///
    /// * range: [0.0, 1.0]
    /// * default: 0.3
    eta: f32,

    /// Minimum loss reduction required to make a further partition on a leaf node of the tree.
    /// The larger, the more conservative the algorithm will be.
    ///
    /// * range: [0,∞]
    /// * default: 0
    gamma: f32,

    /// Maximum depth of a tree, increase this value will make the model more complex / likely to be overfitting.
    /// 0 indicates no limit, limit is required for depth-wise grow policy.
    ///
    /// * range: [0,∞]
    /// * default: 6
    max_depth: u32,

    /// Minimum sum of instance weight (hessian) needed in a child. If the tree partition step results in a leaf
    /// node with the sum of instance weight less than min_child_weight, then the building process will give up
    /// further partitioning.
    /// In linear regression mode, this simply corresponds to minimum number of instances needed to be in each node.
    /// The larger, the more conservative the algorithm will be.
    ///
    /// * range: [0,∞]
    /// * default: 1
    min_child_weight: f32,

    /// Maximum delta step we allow each tree’s weight estimation to be.
    /// If the value is set to 0, it means there is no constraint. If it is set to a positive value,
    /// it can help making the update step more conservative. Usually this parameter is not needed,
    /// but it might help in logistic regression when class is extremely imbalanced.
    /// Set it to value of 1-10 might help control the update.
    ///
    /// * range: [0,∞]
    /// * default: 0
    max_delta_step: f32,

    /// Subsample ratio of the training instance. Setting it to 0.5 means that XGBoost randomly collected half
    /// of the data instances to grow trees and this will prevent overfitting.
    ///
    /// * range: (0, 1]
    /// * default: 1.0
    subsample: f32,

    /// Subsample ratio of columns when constructing each tree.
    ///
    /// * range: (0.0, 1.0]
    /// * default: 1.0
    colsample_bytree: f32,

    /// Subsample ratio of columns for each split, in each level.
    ///
    /// * range: (0.0, 1.0]
    /// * default: 1.0
    colsample_bylevel: f32,

    /// Subsample ratio of columns for each node.
    ///
    /// * range: (0.0, 1.0]
    /// * default: 1.0
    colsample_bynode: f32,

    /// L2 regularization term on weights, increase this value will make model more conservative.
    ///
    /// * default: 1
    lambda: f32,

    /// L1 regularization term on weights, increase this value will make model more conservative.
    ///
    /// * default: 0
    alpha: f32,

    /// The tree construction algorithm used in XGBoost.
    #[builder(default = "TreeMethod::default()")]
    tree_method: TreeMethod,

    /// This is only used for approximate greedy algorithm.
    /// This roughly translated into O(1 / sketch_eps) number of bins. Compared to directly select number of bins,
    /// this comes with theoretical guarantee with sketch accuracy.
    /// Usually user does not have to tune this. but consider setting to a lower number for more accurate enumeration.
    ///
    /// * range: (0.0, 1.0)
    /// * default: 0.03
    sketch_eps: f32,

    /// Control the balance of positive and negative weights, useful for unbalanced classes.
    /// A typical value to consider: sum(negative cases) / sum(positive cases).
    ///
    /// default: 1.0
    scale_pos_weight: f32,

    /// Sequence of tree updaters to run, providing a modular way to construct and to modify the trees.
    ///
    /// * default: vec![]
    updater: Vec<TreeUpdater>,

    /// This is a parameter of the ‘refresh’ updater plugin. When this flag is true, tree leafs as well as tree nodes'
    /// stats are updated. When it is false, only node stats are updated.
    ///
    /// * default: true
    refresh_leaf: bool,

    /// A type of boosting process to run.
    ///
    /// * default: ProcessType::Default
    process_type: ProcessType,

    /// Controls a way new nodes are added to the tree.  Currently supported only if tree_method is set to 'hist'.
    grow_policy: GrowPolicy,

    /// Maximum number of nodes to be added. Only relevant for the `GrowPolicy::LossGuide` grow
    /// policy.
    ///
    /// * default: 0
    max_leaves: u32,

    /// This is only used if 'hist' is specified as tree_method.
    /// Maximum number of discrete bins to bucket continuous features.
    /// Increasing this number improves the optimality of splits at the cost of higher computation time.
    ///
    /// * default: 256
    max_bin: u32,

    /// Number of trees to train in parallel for boosted random forest.
    ///
    /// * default: 1
    num_parallel_tree: u32,

    /// The type of predictor algorithm to use. Provides the same results but allows the use of GPU or CPU.
    ///
    /// * default: [`Predictor::Cpu`](enum.Predictor.html#variant.Cpu)
    predictor: Predictor,
}

impl Default for TreeBoosterParameters {
    fn default() -> Self {
        TreeBoosterParameters {
            eta: 0.3,
            gamma: 0.0,
            max_depth: 6,
            min_child_weight: 1.0,
            max_delta_step: 0.0,
            subsample: 1.0,
            colsample_bytree: 1.0,
            colsample_bylevel: 1.0,
            colsample_bynode: 1.0,
            lambda: 1.0,
            alpha: 0.0,
            tree_method: TreeMethod::default(),
            sketch_eps: 0.03,
            scale_pos_weight: 1.0,
            updater: Vec::new(),
            refresh_leaf: true,
            process_type: ProcessType::default(),
            grow_policy: GrowPolicy::default(),
            max_leaves: 0,
            max_bin: 256,
            num_parallel_tree: 1,
            predictor: Predictor::default(),
        }
    }
}

impl TreeBoosterParameters {
    pub(crate) fn as_string_pairs(&self) -> Vec<(String, String)> {
        let mut v = Vec::new();

        v.push(("booster".to_owned(), "gbtree".to_owned()));

        v.push(("eta".to_owned(), self.eta.to_string()));
        v.push(("gamma".to_owned(), self.gamma.to_string()));
        v.push(("max_depth".to_owned(), self.max_depth.to_string()));
        v.push(("min_child_weight".to_owned(), self.min_child_weight.to_string()));
        v.push(("max_delta_step".to_owned(), self.max_delta_step.to_string()));
        v.push(("subsample".to_owned(), self.subsample.to_string()));
        v.push(("colsample_bytree".to_owned(), self.colsample_bytree.to_string()));
        v.push(("colsample_bylevel".to_owned(), self.colsample_bylevel.to_string()));
        v.push(("colsample_bynode".to_owned(), self.colsample_bynode.to_string()));
        v.push(("lambda".to_owned(), self.lambda.to_string()));
        v.push(("alpha".to_owned(), self.alpha.to_string()));
        v.push(("tree_method".to_owned(), self.tree_method.to_string()));
        v.push(("sketch_eps".to_owned(), self.sketch_eps.to_string()));
        v.push(("scale_pos_weight".to_owned(), self.scale_pos_weight.to_string()));
        v.push(("refresh_leaf".to_owned(), (self.refresh_leaf as u8).to_string()));
        v.push(("process_type".to_owned(), self.process_type.to_string()));
        v.push(("grow_policy".to_owned(), self.grow_policy.to_string()));
        v.push(("max_leaves".to_owned(), self.max_leaves.to_string()));
        v.push(("max_bin".to_owned(), self.max_bin.to_string()));
        v.push(("num_parallel_tree".to_owned(), self.num_parallel_tree.to_string()));
        v.push(("predictor".to_owned(), self.predictor.to_string()));

        // Don't pass anything to XGBoost if the user didn't specify anything.
        // This allows XGBoost to figure it out on it's own, and suppresses the
        // warning message during training.
        // See: https://github.com/davechallis/rust-xgboost/issues/7
        if self.updater.len() != 0
        {
          v.push(("updater".to_owned(), self.updater.iter().map(|u| u.to_string()).collect::<Vec<String>>().join(",")));
        }

        v
    }
}

impl TreeBoosterParametersBuilder {
    fn validate(&self) -> Result<(), String> {
        Interval::new_closed_closed(0.0, 1.0).validate(&self.eta, "eta")?;
        Interval::new_open_closed(0.0, 1.0).validate(&self.subsample, "subsample")?;
        Interval::new_open_closed(0.0, 1.0).validate(&self.colsample_bytree, "colsample_bytree")?;
        Interval::new_open_closed(0.0, 1.0).validate(&self.colsample_bylevel, "colsample_bylevel")?;
        Interval::new_open_closed(0.0, 1.0).validate(&self.colsample_bynode, "colsample_bynode")?;
        Interval::new_open_open(0.0, 1.0).validate(&self.sketch_eps, "sketch_eps")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_params() {
        let p = TreeBoosterParameters::default();
        assert_eq!(p.eta, 0.3);
        let p = TreeBoosterParametersBuilder::default().build().unwrap();
        assert_eq!(p.eta, 0.3);
    }
}
