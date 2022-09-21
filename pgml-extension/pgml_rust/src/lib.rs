extern crate blas;
extern crate openblas_src;
extern crate serde;

use once_cell::sync::Lazy; // 1.3.1
use parking_lot::Mutex;
use pgx::*;
use std::collections::HashMap;
use std::fs;
use xgboost::{Booster, DMatrix};

pub mod api;
pub mod engines;
pub mod orm;
pub mod vectors;

pg_module_magic!();

extension_sql_file!("../sql/schema.sql", name = "schema");

// The mutex is there just to guarantee to Rust that
// there is no concurrent access.
// This space here is connection-specific.
static MODELS: Lazy<Mutex<HashMap<i64, Vec<u8>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[pg_extern]
fn model_predict(model_id: i64, features: Vec<f32>) -> f32 {
    let mut guard = MODELS.lock();

    match guard.get(&model_id) {
        Some(data) => {
            let bst = Booster::load_buffer(data).unwrap();
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

#[pg_extern]
fn model_predict_batch(model_id: i64, features: Vec<f32>, num_rows: i32) -> Vec<f32> {
    let mut guard = MODELS.lock();

    if num_rows < 0 {
        error!("Number of rows has to be greater than 0");
    }

    match guard.get(&model_id) {
        Some(data) => {
            let bst = Booster::load_buffer(data).unwrap();
            let dmat = DMatrix::from_dense(&features, num_rows as usize).unwrap();

            bst.predict(&dmat).unwrap()
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
                    let dmat = DMatrix::from_dense(&features, num_rows as usize).unwrap();

                    bst.predict(&dmat).unwrap()
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
fn load_model(data: Vec<u8>) -> i64 {
    Spi::get_one_with_args::<i64>(
        "INSERT INTO pgml_rust.models (id, algorithm, data) VALUES (DEFAULT, 'xgboost', $1) RETURNING id",
        vec![
            (PgBuiltInOids::BYTEAOID.oid(), data.into_datum()),
        ],
    ).unwrap()
}

/// Load a model into the extension from a file.
#[pg_extern]
fn load_model_from_file(path: String) -> i64 {
    let bytes = fs::read(&path).unwrap();

    Spi::get_one_with_args::<i64>(
        "INSERT INTO pgml_rust.models (id, algorithm, data) VALUES (DEFAULT, 'xgboost', $1) RETURNING id",
        vec![
            (PgBuiltInOids::BYTEAOID.oid(), bytes.into_datum()),
        ],
    ).unwrap()
}

#[pg_extern]
fn delete_model(model_id: i64) {
    Spi::run(&format!(
        "DELETE FROM pgml_rust.models WHERE id = {}",
        model_id
    ));
}

#[pg_extern]
fn dump_model(model_id: i64) -> String {
    let bytes = Spi::get_one_with_args::<Vec<u8>>(
        "SELECT data FROM pgml_rust.models WHERE id = $1",
        vec![(PgBuiltInOids::INT8OID.oid(), model_id.into_datum())],
    );

    match bytes {
        Some(bytes) => match Booster::load_buffer(&bytes) {
            Ok(bst) => bst.dump_model(true, None).unwrap(),
            Err(err) => error!("Could not load XGBoost model: {:?}", err),
        },

        None => error!("Model with id = {} does not exist", model_id),
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {}

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
