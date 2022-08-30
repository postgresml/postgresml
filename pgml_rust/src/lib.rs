use pgx::*;
use xgboost::{parameters, DMatrix, Booster};
use std::path::Path;

pg_module_magic!();

extension_sql_file!("../sql/schema.sql", name = "bootstrap_raw", bootstrap);
extension_sql_file!("../sql/diabetes.sql", name = "diabetes", requires = ["bootstrap_raw"]);

/// Main training function to train an XGBoost model on a dataset.
/// Parameters:
///   - table: name of the table/view
///   - features: array of column names to use as features
///   - label: the name of the target column
///
/// Example:
///
/// ```
/// SELECT * FROM pgml_train('pgml.diabetes', ARRAY['age', 'sex'], 'target');
#[pg_extern]
fn pgml_rust_train(table: String, features: Vec<String>, label: String) {
    let features = features.iter().map(|column| format!("CAST({} AS REAL)", column)).collect::<Vec<String>>();

    let query = format!(
        "SELECT {}, CAST({} AS REAL) FROM {} ORDER BY RANDOM()", features.clone().join(", "), label, table
    );

    let (mut x, mut y, mut num_rows) = (vec![], vec![], 0);

    info!("Fetching data: {}", query);

    Spi::connect(|client| {
        client.select(&query, None, None)
        .for_each(|row| {
            for i in 1..features.len() + 1 {
                x.push(row[i].value::<f32>().unwrap_or(0 as f32));
            }
            y.push(row[features.len() + 1].value::<f32>().unwrap_or(0 as f32));
            num_rows += 1;
        });

        Ok(Some(()))
    });

    let mut dtrain = DMatrix::from_dense(&x, num_rows,).unwrap();
    dtrain.set_labels(&y).unwrap();

    // configure objectives, metrics, etc.
    let learning_params = parameters::learning::LearningTaskParametersBuilder::default()
        .objective(parameters::learning::Objective::RegLinear)
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
    // let evaluation_sets = &[(&dtrain, "train"), (&dtest, "test")];

    // overall configuration for training/evaluation
    let params = parameters::TrainingParametersBuilder::default()
        .dtrain(&dtrain)                         // dataset to train with
        .boost_rounds(2)                         // number of training iterations
        .booster_params(booster_params)          // model parameters
        // .evaluation_sets(Some(evaluation_sets)) // optional datasets to evaluate against in each iteration
        .build().unwrap();

    // train model, and print evaluation data
    let bst = Booster::train(&params).unwrap();

    bst.save(&Path::new("/tmp/xgboost_model.bin")).unwrap();
}

/// Predict a novel data point using the model created by pgml_train.
///
/// Example:
/// ```
/// SELECT * FROM pgml_predict(ARRAY[1, 2, 3]);
#[pg_extern]
fn pgml_rust_predict(features: Vec<f32>) -> f32 {
    let bst = Booster::load(&Path::new("/tmp/xgboost_model.bin")).unwrap();
    let dmat = DMatrix::from_dense(&features, 1).unwrap();

    bst.predict(&dmat).unwrap()[0]
}



#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgx::*;
}

#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
