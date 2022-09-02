extern crate blas;
extern crate openblas_src;

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

/// Predict a novel data point using the model created by pgml_train.
///
/// Example:
/// ```
/// SELECT * FROM pgml_predict(ARRAY[1, 2, 3]);
#[pg_schema]
mod pgml_rust {
    use super::*;

    #[derive(PostgresEnum, Copy, Clone)]
    #[allow(non_camel_case_types)]
    enum Algorithm {
        xgboost,
    }

    #[derive(PostgresEnum, Copy, Clone, PartialEq, Debug)]
    #[allow(non_camel_case_types)]
    enum ProjectTask {
        regression,
        classification,
    }

    impl PartialEq<String> for ProjectTask {
        fn eq(&self, other: &String) -> bool {
            match *self {
                ProjectTask::regression => "regression" == other,
                ProjectTask::classification => "classification" == other,
            }
        }
    }

    impl ProjectTask {
        pub fn to_string(&self) -> String {
            match *self {
                ProjectTask::regression => "regression".to_string(),
                ProjectTask::classification => "classification".to_string(),
            }
        }
    }

    /// Main training function to train an XGBoost model on a dataset.
    ///
    /// Example:
    ///
    /// ```
    /// SELECT * FROM pgml_rust.train('pgml_rust.diabetes', ARRAY['age', 'sex'], 'target');
    #[pg_extern]
    fn train(
        project_name: String,
        task: ProjectTask,
        relation_name: String,
        label: String,
        _algorithm: Algorithm,
        hyperparams: Json,
    ) -> i64 {
        let parts = relation_name
            .split(".")
            .map(|name| name.to_string())
            .collect::<Vec<String>>();

        let (schema_name, table_name) = match parts.len() {
            1 => (String::from("public"), parts[0].clone()),
            2 => (parts[0].clone(), parts[1].clone()),
            _ => error!(
                "Relation name {} is not parsable into schema name and table name",
                relation_name
            ),
        };

        let (mut x, mut y, mut num_rows) = (vec![], vec![], 0);

        let hyperparams = hyperparams.0;

        let (projet_id, project_task) = Spi::get_two_with_args::<i64, String>("INSERT INTO pgml_rust.projects (name, task) VALUES ($1, $2) ON CONFLICT (name) DO UPDATE SET name = $1 RETURNING id, task",
            vec![
                (PgBuiltInOids::TEXTOID.oid(), project_name.clone().into_datum()),
                (PgBuiltInOids::TEXTOID.oid(), task.to_string().into_datum()),
            ]);

        let (projet_id, project_task) = (projet_id.unwrap(), project_task.unwrap());

        if project_task != task.to_string() {
            error!(
                "Project '{}' already exists with a different objective: {}",
                project_name, project_task
            );
        }

        Spi::connect(|client| {
            let mut features = Vec::new();

            client.select("SELECT CAST(column_name AS TEXT) FROM information_schema.columns WHERE table_name = $1 AND table_schema = $2 AND column_name != $3",
                None,
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), table_name.clone().into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), schema_name.into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), label.clone().into_datum()),
                ]))
            .for_each(|row| {
                features.push(row[1].value::<String>().unwrap())
            });

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

            info!("Fetching data: {}", query);

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
            .objective(match task {
                ProjectTask::regression => xgboost::parameters::learning::Objective::RegLinear,
                ProjectTask::classification => {
                    xgboost::parameters::learning::Objective::RegLogistic
                }
            })
            .build()
            .unwrap();

        // configure the tree-based learning model's parameters
        let tree_params = parameters::tree::TreeBoosterParametersBuilder::default()
            .max_depth(match hyperparams.get("max_depth") {
                Some(value) => value.as_u64().unwrap_or(2) as u32,
                None => 2,
            })
            .eta(0.3)
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
            .boost_rounds(match hyperparams.get("n_estimators") {
                Some(value) => value.as_u64().unwrap_or(2) as u32,
                None => 2,
            }) // number of training iterations
            .booster_params(booster_params) // model parameters
            // .evaluation_sets(Some(evaluation_sets)) // optional datasets to evaluate against in each iteration
            .build()
            .unwrap();

        // train model, and print evaluation data
        let bst = match Booster::train(&params) {
            Ok(bst) => bst,
            Err(err) => error!("{}", err),
        };

        let r: u64 = rand::random();
        let path = format!("/tmp/pgml_rust_{}.bin", r);

        bst.save(Path::new(&path)).unwrap();

        let bytes = fs::read(&path).unwrap();

        let model_id = Spi::get_one_with_args::<i64>(
            "INSERT INTO pgml_rust.models (id, project_id, algorithm, data) VALUES (DEFAULT, $1, 'xgboost', $2) RETURNING id",
            vec![
                (PgBuiltInOids::INT8OID.oid(), projet_id.into_datum()),
                (PgBuiltInOids::BYTEAOID.oid(), bytes.into_datum())
            ]
        ).unwrap();

        Spi::get_one_with_args::<i64>(
            "INSERT INTO pgml_rust.deployments (project_id, model_id, strategy) VALUES ($1, $2, 'last_trained') RETURNING id",
            vec![
                (PgBuiltInOids::INT8OID.oid(), projet_id.into_datum()),
                (PgBuiltInOids::INT8OID.oid(), model_id.into_datum()),
            ]
        );

        model_id
    }

    #[pg_extern]
    fn predict(project_name: String, features: Vec<f32>) -> f32 {
        let model_id = Spi::get_one_with_args(
            "SELECT model_id
            FROM pgml_rust.deployments
            INNER JOIN pgml_rust.projects ON
            pgml_rust.deployments.project_id = pgml_rust.projects.id
            AND pgml_rust.projects.name = $1
            ORDER BY pgml_rust.deployments.id DESC LIMIT 1",
            vec![(
                PgBuiltInOids::TEXTOID.oid(),
                project_name.clone().into_datum(),
            )],
        );

        match model_id {
            Some(model_id) => model_predict(model_id, features),
            None => error!("Project '{}' doesn't exist", project_name),
        }
    }

    #[pg_extern]
    fn model_predict(model_id: i64, features: Vec<f32>) -> f32 {
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

    #[pg_extern]
    fn model_predict_batch(model_id: i64, features: Vec<f32>, num_rows: i32) -> Vec<f32> {
        let mut guard = MODELS.lock().unwrap();

        if num_rows < 0 {
            error!("Number of rows has to be greater than 0");
        }

        match guard.get(&model_id) {
            Some(data) => {
                let bst = Booster::load_buffer(&data).unwrap();
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
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_add")]
fn pgml_add_scalar_s(vector: Vec<f32>, addend: f32) -> Vec<f32> {
    vector.as_slice().iter().map(|a| a + addend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_add")]
fn pgml_add_scalar_d(vector: Vec<f64>, addend: f64) -> Vec<f64> {
    vector.as_slice().iter().map(|a| a + addend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_subtract")]
fn pgml_subtract_scalar_s(vector: Vec<f32>, subtahend: f32) -> Vec<f32> {
    vector.as_slice().iter().map(|a| a - subtahend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_subtract")]
fn pgml_subtract_scalar_d(vector: Vec<f64>, subtahend: f64) -> Vec<f64> {
    vector.as_slice().iter().map(|a| a - subtahend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_multiply")]
fn pgml_multiply_scalar_s(vector: Vec<f32>, multiplicand: f32) -> Vec<f32> {
    vector.as_slice().iter().map(|a| a * multiplicand).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_multiply")]
fn pgml_multiply_scalar_d(vector: Vec<f64>, multiplicand: f64) -> Vec<f64> {
    vector.as_slice().iter().map(|a| a * multiplicand).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_divide")]
fn pgml_divide_scalar_s(vector: Vec<f32>, dividend: f32) -> Vec<f32> {
    vector.as_slice().iter().map(|a| a / dividend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_divide")]
fn pgml_divide_scalar_d(vector: Vec<f64>, dividend: f64) -> Vec<f64> {
    vector.as_slice().iter().map(|a| a / dividend).collect()
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_add")]
fn pgml_add_vector_s(vector: Vec<f32>, addend: Vec<f32>) -> Vec<f32> {
    vector.as_slice().iter()
        .zip(addend.as_slice().iter())
        .map(|(a, b)| a + b ).collect()   
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_add")]
fn pgml_add_vector_d(vector: Vec<f64>, addend: Vec<f64>) -> Vec<f64> {
    vector.as_slice().iter()
        .zip(addend.as_slice().iter())
        .map(|(a, b)| a + b ).collect()   
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_subtract")]
fn pgml_subtract_vector_s(vector: Vec<f32>, subtahend: Vec<f32>) -> Vec<f32> {
    vector.as_slice().iter()
        .zip(subtahend.as_slice().iter())
        .map(|(a, b)| a - b ).collect()   
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_subtract")]
fn pgml_subtract_vector_d(vector: Vec<f64>, subtahend: Vec<f64>) -> Vec<f64> {
    vector.as_slice().iter()
        .zip(subtahend.as_slice().iter())
        .map(|(a, b)| a - b ).collect()   
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_multiply")]
fn pgml_multiply_vector_s(vector: Vec<f32>, multiplicand: Vec<f32>) -> Vec<f32> {
    vector.as_slice().iter()
        .zip(multiplicand.as_slice().iter())
        .map(|(a, b)| a * b ).collect()   
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_multiply")]
fn pgml_multiply_vector_d(vector: Vec<f64>, multiplicand: Vec<f64>) -> Vec<f64> {
    vector.as_slice().iter()
        .zip(multiplicand.as_slice().iter())
        .map(|(a, b)| a * b ).collect()   
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_divide")]
fn pgml_divide_vector_s(vector: Vec<f32>, dividend: Vec<f32>) -> Vec<f32> {
    vector.as_slice().iter()
        .zip(dividend.as_slice().iter())
        .map(|(a, b)| a / b ).collect()   
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_divide")]
fn pgml_divide_vector_d(vector: Vec<f64>, dividend: Vec<f64>) -> Vec<f64> {
    vector.as_slice().iter()
        .zip(dividend.as_slice().iter())
        .map(|(a, b)| a / b ).collect()   
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_norm_l0")]
fn pgml_norm_l0_s(vector: Vec<f32>) -> f32 {
    vector.as_slice().iter().map(|a| if *a == 0.0 { 0.0 } else { 1.0 } ).sum()   
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_norm_l0")]
fn pgml_norm_l0_d(vector: Vec<f64>) -> f64 {
    vector.as_slice().iter().map(|a| if *a == 0.0 { 0.0 } else { 1.0 } ).sum()   
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_norm_l1")]
fn pgml_norm_l1_s(vector: Vec<f32>) -> f32 {
    unsafe {
        blas::sasum(vector.len().try_into().unwrap(), vector.as_slice(), 1)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_norm_l1")]
fn pgml_norm_l1_d(vector: Vec<f64>) -> f64 {
    unsafe {
        blas::dasum(vector.len().try_into().unwrap(), vector.as_slice(), 1)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_norm_l2")]
fn pgml_norm_l2_s(vector: Vec<f32>) -> f32 {
    unsafe {
        blas::snrm2(vector.len().try_into().unwrap(), vector.as_slice(), 1)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_norm_l2")]
fn pgml_norm_l2_d(vector: Vec<f64>) -> f64 {
    unsafe {
        blas::dnrm2(vector.len().try_into().unwrap(), vector.as_slice(), 1)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_norm_max")]
fn pgml_norm_max_s(vector: Vec<f32>) -> f32 {
    vector.as_slice().iter().map(|a| a.abs()).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_norm_max")]
fn pgml_norm_max_d(vector: Vec<f64>) -> f64 {
    vector.as_slice().iter().map(|a| a.abs()).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_normalize_l1")]
fn pgml_normalize_l1_s(vector: Vec<f32>) -> Vec<f32> {
    let norm: f32;
    unsafe {
        norm = blas::sasum(vector.len().try_into().unwrap(), vector.as_slice(), 1);
    }
    pgml_divide_scalar_s(vector, norm)
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_normalize_l1")]
fn pgml_normalize_l1_d(vector: Vec<f64>) -> Vec<f64> {
    let norm: f64;
    unsafe {
        norm = blas::dasum(vector.len().try_into().unwrap(), vector.as_slice(), 1);
    }
    pgml_divide_scalar_d(vector, norm)
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_normalize_l2")]
fn pgml_normalize_l2_s(vector: Vec<f32>) -> Vec<f32> {
    let norm: f32;
    unsafe {
        norm = blas::snrm2(vector.len().try_into().unwrap(), vector.as_slice(), 1);
    }
    pgml_divide_scalar_s(vector, norm)
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_normalize_l2")]
fn pgml_normalize_l2_d(vector: Vec<f64>) -> Vec<f64> {
    let norm: f64;
    unsafe {
        norm = blas::dnrm2(vector.len().try_into().unwrap(), vector.as_slice(), 1);
    }
    pgml_divide_scalar_d(vector, norm)
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_normalize_max")]
fn pgml_normalize_max_s(vector: Vec<f32>) -> Vec<f32> {
    let norm = vector.as_slice().iter().map(|a| a.abs()).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    pgml_divide_scalar_s(vector, norm)
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_normalize_max")]
fn pgml_normalize_max_d(vector: Vec<f64>) -> Vec<f64> {
    let norm = vector.as_slice().iter().map(|a| a.abs()).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    pgml_divide_scalar_d(vector, norm)
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_distance_l1")]
fn pgml_distance_l1_s(vector: Vec<f32>, other: Vec<f32>) -> f32 {
    vector.as_slice().iter()
        .zip(other.as_slice().iter())
        .map(|(a, b)| (a - b).abs() ).sum()
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_distance_l1")]
fn pgml_distance_l1_d(vector: Vec<f64>, other: Vec<f64>) -> f64 {
    vector.as_slice().iter()
        .zip(other.as_slice().iter())
        .map(|(a, b)| (a - b).abs() ).sum()
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_distance_l2")]
fn pgml_distance_l2_s(vector: Vec<f32>, other: Vec<f32>) -> f32 {
    vector.as_slice().iter()
        .zip(other.as_slice().iter())
        .map(|(a, b)| (a - b).powf(2.0) ).sum::<f32>().sqrt()
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_distance_l2")]
fn pgml_distance_l2_d(vector: Vec<f64>, other: Vec<f64>) -> f64 {
    vector.as_slice().iter()
        .zip(other.as_slice().iter())
        .map(|(a, b)| (a - b).powf(2.0) ).sum::<f64>().sqrt()
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_dot_product")]
fn pgml_dot_product_s(vector: Vec<f32>, other: Vec<f32>) -> f32 {
    unsafe {
        blas::sdot(vector.len().try_into().unwrap(), vector.as_slice(), 1, other.as_slice(), 1)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_dot_product")]
fn pgml_dot_product_d(vector: Vec<f64>, other: Vec<f64>) -> f64 {
    unsafe {
        blas::ddot(vector.len().try_into().unwrap(), vector.as_slice(), 1, other.as_slice(), 1)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_cosine_similarity")]
fn pgml_cosine_similarity_s(vector: Vec<f32>, other: Vec<f32>) -> f32 {
    unsafe {
        let dot = blas::sdot(vector.len().try_into().unwrap(), vector.as_slice(), 1, other.as_slice(), 1);
        let a_norm = blas::snrm2(vector.len().try_into().unwrap(), vector.as_slice(), 1);
        let b_norm = blas::snrm2(other.len().try_into().unwrap(), other.as_slice(), 1);
        dot / (a_norm * b_norm)
    }
}

#[pg_extern(immutable, parallel_safe, strict, name="pgml_cosine_similarity")]
fn pgml_cosine_similarity_d(vector: Vec<f64>, other: Vec<f64>) -> f64 {
    unsafe {
        let dot = blas::ddot(vector.len().try_into().unwrap(), vector.as_slice(), 1, other.as_slice(), 1);
        let a_norm = blas::dnrm2(vector.len().try_into().unwrap(), vector.as_slice(), 1);
        let b_norm = blas::dnrm2(other.len().try_into().unwrap(), other.as_slice(), 1);
        dot / (a_norm * b_norm)
    }
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
