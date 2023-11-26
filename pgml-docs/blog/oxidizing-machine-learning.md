---
description: >-
  PostgresML's architecture gives it a huge performance advantage over
  traditional deployments when it comes to latency, throughput and memory
  utilization.
---

# Oxidizing Machine Learning

<div align="left">

<figure><img src=".gitbook/assets/lev.jpg" alt="Author" width="100"><figcaption></figcaption></figure>

</div>

Lev Kokotov

September 7, 2022

Machine learning in Python can be hard to deploy at scale. We all love Python, but it's no secret that its overhead is large:

* Load data from large CSV files
* Do some post-processing with NumPy
* Move and join data into a Pandas dataframe
* Load data into the algorithm

Each step incurs at least one copy of the data in memory; 4x storage and compute cost for training a model sounds inefficient, but when you add Python's memory allocation, the price tag increases exponentially.

Even if you could find the money to pay for the compute needed, fitting the dataset we want into the RAM we have becomes difficult.

The status quo needs a shake up, and along came Rust.

## The State of ML in Rust

Doing machine learning in anything but Python sounds wild, but if one looks under the hood, ML algorithms are mostly written in C++: `libtorch` (Torch), XGBoost, large parts of Tensorflow, `libsvm` (Support Vector Machines), and the list goes on. A linear regression can be (and is) written in about 10 lines of for-loops.

It then should come to no surprise that the Rust ML community is alive, and doing well:

* SmartCore[^1] is rivaling Scikit for commodity algorithms
* XGBoost bindings[^2] work great for gradient boosted trees
* Torch bindings[^3] are first class for building any kind of neural network
* Tensorflow bindings[^4] are also in the mix, although parts of them are still Python (e.g. Keras)

If you start missing NumPy, don't worry, the Rust version[^5] has got you covered, and the list of available tools keeps growing.

When you only need 4 bytes to represent a floating point instead of Python's 26 bytes[^6], suddenly you can do more.

## XGBoost, Rustified

Let's do a quick example to illustrate our point.

XGBoost is a popular decision tree algorithm which uses gradient boosting, a fancy optimization technique, to train algorithms on data that could confuse simpler linear models. It comes with a Python interface, which calls into its C++ primitives, but now, it has a Rust interface as well.

_Cargo.toml_

```toml
[dependencies]
xgboost = "0.1"
```

_src/main.rs_

```rust
use xgboost::{parameters, Booster, DMatrix};

fn main() {
    // Data is read directly into the C++ data structure
    let train = DMatrix::load("train.txt").unwrap();
    let test = DMatrix::load("test.txt").unwrap();

    // Task (regression or classification)
    let learning_params = parameters::learning::LearningTaskParametersBuilder::default()
        .objective(parameters::learning::Objective::BinaryLogistic)
        .build()
        .unwrap();

    // Tree parameters (e.g. depth)
    let tree_params = parameters::tree::TreeBoosterParametersBuilder::default()
        .max_depth(2)
        .eta(1.0)
        .build()
        .unwrap();

    // Gradient boosting parameters
    let booster_params = parameters::BoosterParametersBuilder::default()
        .booster_type(parameters::BoosterType::Tree(tree_params))
        .learning_params(learning_params)
        .build()
        .unwrap();

    // Train on train data, test accuracy on test data
    let evaluation_sets = &[(&train, "train"), (&test, "test")];

    // Final algorithm configuration
    let params = parameters::TrainingParametersBuilder::default()
        .dtrain(&train)
        .boost_rounds(2) // n_estimators
        .booster_params(booster_params)
        .evaluation_sets(Some(evaluation_sets))
        .build()
        .unwrap();

    // Train the model
    let model = Booster::train(&params).unwrap();

    // Save and load later in any language that has XGBoost bindings
    model.save("/tmp/xgboost_model.bin").unwrap();
}
```

Example created from `rust-xgboost` documentation and my own experiments.

That's it! You just trained an XGBoost model in Rust, in just a few lines of efficient and ergonomic code.

Unlike Python, Rust compiles and verifies your code, so you'll know that it's likely to work before you even run it. When it can take several hours to train a model, it's great to know that you don't have a syntax error on your last line.

[^1]: [SmartCore](https://smartcorelib.org/)

[^2]: [XGBoost bindings](https://github.com/davechallis/rust-xgboost)

[^3]: [Torch bindings](https://github.com/LaurentMazare/tch-rs)

[^4]: [Tensorflow bindings](https://github.com/tensorflow/rust)

[^5]: [rust-ndarray](https://github.com/rust-ndarray/ndarray)

[^6]: [Python floating points](https://github.com/python/cpython/blob/e42b705188271da108de42b55d9344642170aa2b/Include/floatobject.h#L15)
