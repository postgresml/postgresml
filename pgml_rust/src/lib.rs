use once_cell::sync::Lazy; // 1.3.1
use pgx::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use xgboost::{parameters, Booster, DMatrix};

pg_module_magic!();

extension_sql_file!("../sql/schema.sql", name = "bootstrap_raw", bootstrap);
extension_sql_file!(
    "../sql/diabetes.sql",
    name = "diabetes",
    requires = ["bootstrap_raw"]
);

// The mutex is there just to guarantee to Rust that
// there is no concurrent access.
// This space here is connection-specific.
static MODELS: Lazy<Mutex<HashMap<i64, Vec<u8>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Main training function to train an XGBoost model on a dataset.
///
/// Example:
///
/// ```
/// SELECT * FROM pgml_train('pgml_rust.diabetes', ARRAY['age', 'sex'], 'target');
#[pg_extern]
fn pgml_rust_train(relation_name: String, features: Vec<String>, label: String) -> i64 {
    let features = features
        .into_iter()
        .map(|column| format!("CAST({} AS REAL)", column))
        .collect::<Vec<String>>();

    let query = format!(
        "SELECT {}, CAST({} AS REAL) FROM {} ORDER BY RANDOM()",
        features.clone().join(", "),
        label,
        relation_name
    );

    let (mut x, mut y, mut num_rows) = (vec![], vec![], 0);

    info!("Fetching data: {}", query);

    Spi::connect(|client| {
        client.select(&query, None, None).for_each(|row| {
            // Postgres arrays start at one and for some reason
            // so do these tuple indexes.
            for i in 1..features.len() + 1 {
                x.push(row[i].value::<f32>().unwrap_or(0 as f32));
            }
            y.push(row[features.len() + 1].value::<f32>().unwrap_or(0 as f32));
            num_rows += 1;
        });

        Ok(Some(()))
    });

    let mut dtrain = DMatrix::from_dense(&x, num_rows).unwrap();
    dtrain.set_labels(&y).unwrap();

    // configure objectives, metrics, etc.
    let learning_params = parameters::learning::LearningTaskParametersBuilder::default()
        .objective(parameters::learning::Objective::RegLinear)
        .build()
        .unwrap();

    // configure the tree-based learning model's parameters
    let tree_params = parameters::tree::TreeBoosterParametersBuilder::default()
        .max_depth(2)
        .eta(1.0)
        .build()
        .unwrap();

    // overall configuration for Booster
    let booster_params = parameters::BoosterParametersBuilder::default()
        .booster_type(parameters::BoosterType::Tree(tree_params))
        .learning_params(learning_params)
        .verbose(true)
        .build()
        .unwrap();

    // specify datasets to evaluate against during training
    // let evaluation_sets = &[(&dtrain, "train"), (&dtest, "test")];

    // overall configuration for training/evaluation
    let params = parameters::TrainingParametersBuilder::default()
        .dtrain(&dtrain) // dataset to train with
        .boost_rounds(2) // number of training iterations
        .booster_params(booster_params) // model parameters
        // .evaluation_sets(Some(evaluation_sets)) // optional datasets to evaluate against in each iteration
        .build()
        .unwrap();

    // train model, and print evaluation data
    let bst = Booster::train(&params).unwrap();

    let r: i64 = rand::random();
    let path = format!("/tmp/pgml_rust_{}.bin", r);

    bst.save(Path::new(&path)).unwrap();

    let bytes = fs::read(&path).unwrap();

    Spi::get_one_with_args::<i64>(
        "INSERT INTO pgml_rust.models (id, algorithm, data) VALUES (DEFAULT, 'xgboost', $1) RETURNING id",
        vec![
            (PgBuiltInOids::BYTEAOID.oid(), bytes.into_datum())
        ]
    ).unwrap()
}

/// Predict a novel data point using the model created by pgml_train.
///
/// Example:
/// ```
/// SELECT * FROM pgml_predict(ARRAY[1, 2, 3]);
#[pg_extern]
fn pgml_rust_predict(model_id: i64, features: Vec<f32>) -> f32 {
    let mut guard = MODELS.lock().unwrap();

    match guard.get(&model_id) {
        Some(data) => {
            let bst = Booster::load_buffer(&data).unwrap();
            let dmat = DMatrix::from_dense(&features, 1).unwrap();

            bst.predict(&dmat).unwrap()[0]
        }

        None => {
            match Spi::get_one_with_args::<Vec<u8>>(
                "SELECT data FROM pgml_rust.models WHERE id = $1",
                vec![(PgBuiltInOids::INT8OID.oid(), model_id.into_datum())],
            ) {
                Some(data) => {
                    info!("Model cache cold, loading from \"pgml_rust\".\"models\"");

                    guard.insert(model_id, data.clone());
                    let bst = Booster::load_buffer(&data).unwrap();
                    let dmat = DMatrix::from_dense(&features, 1).unwrap();

                    bst.predict(&dmat).unwrap()[0]
                }
                None => {
                    error!("No model with id = {} found", model_id);
                }
            }
        }
    }
}

/// Load a model into the extension. The model is saved in our table,
/// which is then replicated to replicas for load balancing.
#[pg_extern]
fn pgml_rust_load_model(data: Vec<u8>) -> i64 {
    Spi::get_one_with_args::<i64>(
        "INSERT INTO pgml_rust.models (id, algorithm, data) VALUES (DEFAULT, 'xgboost', $1) RETURNING id",
        vec![
            (PgBuiltInOids::BYTEAOID.oid(), data.into_datum()),
        ],
    ).unwrap()
}

/// Load a model into the extension from a file.
#[pg_extern]
fn pgml_rust_load_model_from_file(path: String) -> i64 {
    let bytes = fs::read(&path).unwrap();

    Spi::get_one_with_args::<i64>(
        "INSERT INTO pgml_rust.models (id, algorithm, data) VALUES (DEFAULT, 'xgboost', $1) RETURNING id",
        vec![
            (PgBuiltInOids::BYTEAOID.oid(), bytes.into_datum()),
        ],
    ).unwrap()
}

#[pg_extern]
fn pgml_rust_delete_model(model_id: i64) {
    Spi::run(&format!(
        "DELETE FROM pgml_rust.models WHERE id = {}",
        model_id
    ));
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
