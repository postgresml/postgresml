extern crate xgboost;
extern crate reqwest;
extern crate env_logger;
#[macro_use]
extern crate log;

use std::path::Path;
use std::io::{BufRead, BufReader, BufWriter};
use std::fs::File;
use xgboost::{DMatrix, Booster};
use xgboost::parameters::{self, tree, learning::Objective};



fn main() {
    // initialise logging, run with e.g. RUST_LOG=xgboost_multiclass_classification_example=debug
    env_logger::init();

    // download training data, if not already present locally
    download_dataset("dermatology.data");

    // load train and test matrices from text files (in LibSVM format).
    let (dtrain, dtest) = load_train_test_dmats("dermatology.data");

    // evaluate against both datasets during training
    let eval_sets = &[(&dtrain, "train"), (&dtest, "test")];

    // configure learning objective to use multiclass softmax with 6 classes
    let learning_params = parameters::learning::LearningTaskParametersBuilder::default()
        .objective(Objective::MultiSoftmax(6))
        .build().unwrap();

    // configure tree gradient boosting parameters
    let tree_params = tree::TreeBoosterParametersBuilder::default()
        .eta(0.1)
        .max_depth(6)
        .build().unwrap();

    // configure booster
    let booster_params = parameters::BoosterParametersBuilder::default()
        .booster_type(parameters::BoosterType::Tree(tree_params))
        .learning_params(learning_params)
        .threads(Some(4))
        .build().unwrap();

    // configure the training run
    let training_params = parameters::TrainingParametersBuilder::default()
        .dtrain(&dtrain)
        .booster_params(booster_params)
        .boost_rounds(5)
        .evaluation_sets(Some(eval_sets))
        .build().unwrap();

    // train a new booster model with given parameters, printing results on evaluation sets
    let booster = Booster::train(&training_params).unwrap();

    let y_true = dtest.get_labels().unwrap();
    let y_pred = booster.predict(&dtest).unwrap();
    let num_errors: u32 = y_true.iter()
        .zip(y_pred.iter())
        .map(|(y1, y2)| if y1 != y2 { 1 } else { 0 })
        .sum();
    let error_rate = num_errors as f32 / y_true.len() as f32;
    println!("Test error using softmax: {}", error_rate);
}

fn download_dataset<P: AsRef<Path>>(dst: P) {
    let url = "https://archive.ics.uci.edu/ml/machine-learning-databases/dermatology/dermatology.data";
    let dst = dst.as_ref();
    if dst.exists() {
        debug!("Training dataset '{}' found", dst.display());
        return;
    }

    debug!("Fetching training dataset from {}", url);
    let mut response = reqwest::blocking::get(url).expect("failed to download training set data");

    let file = File::create(dst).expect(&format!("failed to create file {}", dst.display()));
    let mut writer = BufWriter::new(file);
    response.copy_to(&mut writer).expect(&format!("failed to write to {}", dst.display()));
}

fn load_train_test_dmats<P: AsRef<Path>>(src: P) -> (DMatrix, DMatrix) {
    let src = src.as_ref();
    let file = File::open(src).expect(&format!("failed to open {}", src.display()));
    let reader = BufReader::new(file);

    let mut x: Vec<Vec<f32>> = Vec::new();
    let mut y: Vec<f32> = Vec::new();
    for line in reader.lines() {
        let line = line.unwrap();
        let cols: Vec<f32> = line.split(',')
            .enumerate()
            .map(|(col_num, value)| {
                match col_num {
                    // assign value to column which can contain missing data
                    33 => if value == "?" { 1.0 } else { 0.0 },

                    // convert class number from string -> zero based class ID float
                    34 => value.parse::<f32>().unwrap() - 1.0,

                    // convert column values from string -> float
                    _  => value.parse::<f32>().unwrap()
                }
            })
            .collect();

        // skip column 33
        x.push(cols[0..33].to_vec());

        // final column contains class
        y.push(cols[34]);
    }

    let num_rows = x.len();
    let num_cols = x[0].len();

    let train_size = (0.7 * num_rows as f32) as usize;
    let test_size = num_rows - train_size;

    debug!("Parsed {}x{} matrix from dataset", num_rows, num_cols);

    // flatten into 1D vector
    let x_train: Vec<f32> = x[0..train_size].into_iter()
        .flat_map(|row| row.iter().cloned())
        .collect();
    let mut dtrain = DMatrix::from_dense(&x_train, train_size).unwrap();
    dtrain.set_labels(&y[0..train_size]).unwrap();
    let x_test: Vec<f32> = x[train_size..].into_iter()
        .flat_map(|row| row.iter().cloned())
        .collect();
    let mut dtest = DMatrix::from_dense(&x_test, test_size).unwrap();
    dtest.set_labels(&y[train_size..]).unwrap();

    (dtrain, dtest)
}
