extern crate blas;
extern crate openblas_src;
extern crate rmp_serde;
extern crate serde;

use ndarray::Array;
use once_cell::sync::Lazy; // 1.3.1
use pgx::*;
use rmp_serde::Serializer;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use xgboost::{parameters, Booster, DMatrix};

pub mod model;
pub mod vectors;

pg_module_magic!();

extension_sql_file!("../sql/schema.sql", name = "bootstrap_raw");
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
#[derive(PostgresEnum, Copy, Clone, PartialEq)]
#[allow(non_camel_case_types)]
enum OldAlgorithm {
    linear,
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
fn train_old(
    project_name: String,
    task: ProjectTask,
    relation_name: String,
    label: String,
    algorithm: OldAlgorithm,
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

    let (mut x, mut y, mut num_rows, mut num_features) = (vec![], vec![], 0, 0);

    let hyperparams = hyperparams.0;

    let (project_id, project_task) = Spi::get_two_with_args::<i64, String>("INSERT INTO pgml_rust.projects (name, task) VALUES ($1, $2) ON CONFLICT (name) DO UPDATE SET name = $1 RETURNING id, task",
        vec![
            (PgBuiltInOids::TEXTOID.oid(), project_name.clone().into_datum()),
            (PgBuiltInOids::TEXTOID.oid(), task.to_string().into_datum()),
        ]);

    let (project_id, project_task) = (project_id.unwrap(), project_task.unwrap());

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

        // TODO: Optimize for SIMD
        client.select(&query, None, None).for_each(|row| {
            // Postgres arrays start at one and for some reason
            // so do these tuple indexes.
            for i in 1..features.len() + 1 {
                x.push(row[i].value::<f32>().unwrap_or(0 as f32));
            }
            y.push(row[features.len() + 1].value::<f32>().unwrap_or(0 as f32));
            num_rows += 1;
        });

        num_features = features.len();

        Ok(Some(()))
    });

    // todo parameterize test split instead of 0.5
    let test_rows = (num_rows as f32 * 0.5).round() as usize;
    let train_rows = num_rows - test_rows;

    if algorithm == OldAlgorithm::xgboost {
        let mut dtrain = DMatrix::from_dense(&x[..train_rows * num_features], train_rows).unwrap();
        let mut dtest = DMatrix::from_dense(&x[train_rows * num_features..], test_rows).unwrap();
        dtrain.set_labels(&y[..train_rows]).unwrap();
        dtest.set_labels(&y[train_rows..]).unwrap();

        // specify datasets to evaluate against during training
        let evaluation_sets = &[(&dtrain, "train"), (&dtest, "test")];

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

        // overall configuration for training/evaluation
        let params = parameters::TrainingParametersBuilder::default()
            .dtrain(&dtrain) // dataset to train with
            .boost_rounds(match hyperparams.get("n_estimators") {
                Some(value) => value.as_u64().unwrap_or(2) as u32,
                None => 2,
            }) // number of training iterations
            .booster_params(booster_params) // model parameters
            .evaluation_sets(Some(evaluation_sets)) // optional datasets to evaluate against in each iteration
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
                (PgBuiltInOids::INT8OID.oid(), project_id.into_datum()),
                (PgBuiltInOids::BYTEAOID.oid(), bytes.into_datum())
            ]
        ).unwrap();

        Spi::get_one_with_args::<i64>(
            "INSERT INTO pgml_rust.deployments (project_id, model_id, strategy) VALUES ($1, $2, 'last_trained') RETURNING id",
            vec![
                (PgBuiltInOids::INT8OID.oid(), project_id.into_datum()),
                (PgBuiltInOids::INT8OID.oid(), model_id.into_datum()),
            ]
        );
        model_id
    } else {
        let x_train = Array::from_shape_vec(
            (train_rows, num_features),
            x[..train_rows * num_features].to_vec(),
        )
        .unwrap();
        let x_test = Array::from_shape_vec(
            (test_rows, num_features),
            x[train_rows * num_features..].to_vec(),
        )
        .unwrap();
        let y_train = Array::from_shape_vec(train_rows, y[..train_rows].to_vec()).unwrap();
        let y_test = Array::from_shape_vec(test_rows, y[train_rows..].to_vec()).unwrap();
        if task == ProjectTask::regression {
            let estimator = smartcore::linear::linear_regression::LinearRegression::fit(
                &x_train,
                &y_train,
                Default::default(),
            )
            .unwrap();
            save(estimator, x_test, y_test, algorithm, project_id)
        } else if task == ProjectTask::classification {
            let estimator = smartcore::linear::logistic_regression::LogisticRegression::fit(
                &x_train,
                &y_train,
                Default::default(),
            )
            .unwrap();
            save(estimator, x_test, y_test, algorithm, project_id)
        } else {
            0
        }
    }
}

fn save<
    E: serde::Serialize + smartcore::api::Predictor<X, Y> + std::fmt::Debug,
    N: smartcore::math::num::RealNumber,
    X,
    Y: std::fmt::Debug + smartcore::linalg::BaseVector<N>,
>(
    estimator: E,
    x_test: X,
    y_test: Y,
    algorithm: OldAlgorithm,
    project_id: i64,
) -> i64 {
    let y_hat = estimator.predict(&x_test).unwrap();

    let mut buffer = Vec::new();
    estimator
        .serialize(&mut Serializer::new(&mut buffer))
        .unwrap();
    info!("bin {:?}", buffer);
    info!("estimator: {:?}", estimator);
    info!("y_hat: {:?}", y_hat);
    info!("y_test: {:?}", y_test);
    info!("r2: {:?}", smartcore::metrics::r2(&y_test, &y_hat));
    info!(
        "mean squared error: {:?}",
        smartcore::metrics::mean_squared_error(&y_test, &y_hat)
    );

    let mut buffer = Vec::new();
    estimator
        .serialize(&mut Serializer::new(&mut buffer))
        .unwrap();

    let model_id = Spi::get_one_with_args::<i64>(
        "INSERT INTO pgml_rust.models (id, project_id, algorithm, data) VALUES (DEFAULT, $1, $2, $3) RETURNING id",
        vec![
            (PgBuiltInOids::INT8OID.oid(), project_id.into_datum()),
            (PgBuiltInOids::INT8OID.oid(), algorithm.into_datum()),
            (PgBuiltInOids::BYTEAOID.oid(), buffer.into_datum())
        ]
    ).unwrap();

    Spi::get_one_with_args::<i64>(
        "INSERT INTO pgml_rust.deployments (project_id, model_id, strategy) VALUES ($1, $2, 'last_trained') RETURNING id",
        vec![
            (PgBuiltInOids::INT8OID.oid(), project_id.into_datum()),
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
