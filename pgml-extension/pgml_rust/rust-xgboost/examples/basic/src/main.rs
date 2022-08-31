extern crate xgboost;
extern crate sprs;
extern crate env_logger;

use std::io::{BufRead, BufReader};
use std::fs::File;
use xgboost::{parameters, DMatrix, Booster};

fn main() {
    // initialise logging, run with e.g. RUST_LOG=xgboost=debug to see more details
    env_logger::init();

    // load train and test matrices from text files (in LibSVM format).
    println!("Loading train and test matrices...");
    let dtrain = DMatrix::load("../../xgboost-sys/xgboost/demo/data/agaricus.txt.train").unwrap();
    println!("Train matrix: {}x{}", dtrain.num_rows(), dtrain.num_cols());
    let dtest = DMatrix::load("../../xgboost-sys/xgboost/demo/data/agaricus.txt.test").unwrap();
    println!("Test matrix: {}x{}", dtest.num_rows(), dtest.num_cols());

    // configure objectives, metrics, etc.
    let learning_params = parameters::learning::LearningTaskParametersBuilder::default()
        .objective(parameters::learning::Objective::BinaryLogistic)
        .build().unwrap();

    // configure the tree-based learning model's parameters
    let tree_params = parameters::tree::TreeBoosterParametersBuilder::default()
            .max_depth(2)
            .eta(1.0)
            .build().unwrap();

    // overall configuration for Booster
    let booster_params = parameters::BoosterParametersBuilder::default()
        .booster_type(parameters::BoosterType::Tree(tree_params))
        .learning_params(learning_params)
        .verbose(true)
        .build().unwrap();

    // specify datasets to evaluate against during training
    let evaluation_sets = [(&dtest, "test"), (&dtrain, "train")];

    // overall configuration for training/evaluation
    let training_params = parameters::TrainingParametersBuilder::default()
        .dtrain(&dtrain)                         // dataset to train with
        .boost_rounds(2)                         // number of training iterations
        .booster_params(booster_params)          // model parameters
        .evaluation_sets(Some(&evaluation_sets)) // optional datasets to evaluate against in each iteration
        .build().unwrap();

    // train booster model, and print evaluation metrics
    println!("\nTraining tree booster...");
    let booster = Booster::train(&training_params).unwrap();

    // get predictions probabilities for given matrix
    let preds = booster.predict(&dtest).unwrap();

    // get predicted labels for each test example (i.e. 0 or 1)
    println!("\nChecking predictions...");
    let labels = dtest.get_labels().unwrap();
    println!("First 3 predicted labels: {} {} {}", labels[0], labels[1], labels[2]);

    // print error rate
    let num_correct: usize = preds.iter()
        .map(|&v| if v > 0.5 { 1 } else { 0 })
        .sum();
    println!("error={} ({}/{} correct)", num_correct as f32 / preds.len() as f32, num_correct, preds.len());

    // save and load model file
    println!("\nSaving and loading Booster model...");
    booster.save("xgb.model").unwrap();
    let booster = Booster::load("xgb.model").unwrap();
    let preds2 = booster.predict(&dtest).unwrap();
    assert_eq!(preds, preds2);

    // save and load data matrix file
    println!("\nSaving and loading matrix data...");
    dtest.save("test.dmat").unwrap();
    let dtest2 = DMatrix::load("test.dmat").unwrap();
    assert_eq!(booster.predict(&dtest2).unwrap(), preds);

    // error handling example
    println!("\nError message example...");
    let result = Booster::load("/does/not/exist");
    match result {
        Ok(_booster) => (),
        Err(err) => println!("Got expected error: {}", err),
    }

    // sparse matrix usage
    println!("\nSparse matrix construction...");

    // f32 label for each row of data
    let mut labels = Vec::new();

    // construct sparse matrix in triplet format, then convert to CSR/CSC later
    let mut rows = Vec::new();
    let mut cols = Vec::new();
    let mut data = Vec::new();

    let reader = BufReader::new(File::open("../../xgboost-sys/xgboost/demo/data/agaricus.txt.train").unwrap());
    let mut current_row = 0;
    for line in reader.lines() {
        let line = line.unwrap();
        let sample: Vec<&str> = line.split_whitespace().collect();
        labels.push(sample[0].parse::<f32>().unwrap());

        for entry in &sample[1..] {
            let pair: Vec<&str> = entry.split(':').collect();
            rows.push(current_row);
            cols.push(pair[0].parse::<usize>().unwrap());
            data.push(pair[1].parse::<f32>().unwrap());
        }

        current_row += 1;
    }

    // work out size of sparse matrix from max row/col values
    let shape = ((*rows.iter().max().unwrap() + 1) as usize,
                 (*cols.iter().max().unwrap() + 1) as usize);
    let num_col = Some((*cols.iter().max().unwrap() + 1) as usize);
    let triplet_mat = sprs::TriMatBase::from_triplets(shape, rows, cols, data);
    let csr_mat = triplet_mat.to_csr();

    let indices: Vec<usize> = csr_mat.indices().into_iter().map(|i| *i as usize).collect();
    let mut dtrain = DMatrix::from_csr(csr_mat.indptr().raw_storage(), &indices, csr_mat.data(), num_col).unwrap();
    dtrain.set_labels(&labels).unwrap();

    let training_params = parameters::TrainingParametersBuilder::default().dtrain(&dtrain).build().unwrap();
    let _ = Booster::train(&training_params).unwrap();
}
