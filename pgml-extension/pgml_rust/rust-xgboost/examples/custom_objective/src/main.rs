extern crate xgboost;
extern crate ndarray;

use xgboost::{parameters, DMatrix, Booster};

fn main() {
    // load train and test matrices from text files (in LibSVM format)
    println!("Custom objective example...");
    let dtrain = DMatrix::load("../../xgboost-sys/xgboost/demo/data/agaricus.txt.train").unwrap();
    let dtest = DMatrix::load("../../xgboost-sys/xgboost/demo/data/agaricus.txt.test").unwrap();

    // specify datasets to evaluate against during training
    let evaluation_sets = [(&dtest, "test"), (&dtrain, "train")];

    // define custom objective function
    fn log_reg_obj(preds: &[f32], dtrain: &DMatrix) -> (Vec<f32>, Vec<f32>) {
        let mut preds = ndarray::Array1::from_vec(preds.to_vec());
        preds.map_inplace(|x| *x = (-*x).exp());
        preds = 1.0 / (1.0 + preds);

        let labels = ndarray::Array1::from_vec(dtrain.get_labels().unwrap().to_vec());
        let gradient = &preds - &labels;
        let hessian = &preds * &(1.0 - &preds);

        (gradient.to_vec(), hessian.to_vec())
    }

    // define custom evaluation function
    fn eval_error(preds: &[f32], dtrain: &DMatrix) -> f32 {
        let labels = dtrain.get_labels().unwrap();
        let preds = ndarray::Array1::from_vec(preds.to_vec());
        let mut num_incorrect = 0;
        for (label, pred) in labels.iter().zip(preds.iter()) {
            let pred = if *pred > 0.0 { 1.0 } else { 0.0 };
            if pred != *label  {
                num_incorrect += 1;
            }
        }
        num_incorrect as f32 / labels.len() as f32
    }

    let tree_params = parameters::tree::TreeBoosterParametersBuilder::default()
            .max_depth(2)
            .eta(1.0)
            .build().unwrap();

    // overall configuration for Booster
    let booster_params = parameters::BoosterParametersBuilder::default()
        .learning_params(parameters::learning::LearningTaskParameters::default())
        .booster_type(parameters::BoosterType::Tree(tree_params))
        .build().unwrap();

    let training_params = parameters::TrainingParametersBuilder::default()
        .dtrain(&dtrain)
        .booster_params(booster_params)
        .boost_rounds(2)
        .evaluation_sets(Some(&evaluation_sets))
        .custom_objective_fn(Some(log_reg_obj))
        .custom_evaluation_fn(Some(eval_error))
        .build().unwrap();

    // train booster model, and print evaluation metrics
    println!("\nTraining tree booster...");
    let bst = Booster::train(&training_params).unwrap();

    // get predictions probabilities for given matrix
    let preds = bst.predict(&dtest).unwrap();

    // get predicted labels for each test example (i.e. 0 or 1)
    println!("\nChecking predictions...");
    let labels = dtest.get_labels().unwrap();
    println!("First 3 predicated labels: {} {} {}", labels[0], labels[1], labels[2]);

    // print error rate
    let num_correct: usize = preds.iter()
        .map(|&v| if v > 0.5 { 1 } else { 0 })
        .sum();
    println!("error={} ({}/{} correct)", num_correct as f32 / preds.len() as f32, num_correct, preds.len());
}
