use std::str::FromStr;

use ndarray::{Array1, Array2};
use pgx::*;
use serde_json::json;
use xgboost::{parameters, Booster, DMatrix};

use crate::orm::estimator::BoosterBox;
use crate::orm::Algorithm;
use crate::orm::Dataset;
use crate::orm::Estimator;
use crate::orm::Project;
use crate::orm::Search;
use crate::orm::Snapshot;
use crate::orm::Task;

#[derive(Debug)]
pub struct Model {
    pub id: i64,
    pub project_id: i64,
    pub snapshot_id: i64,
    pub algorithm: Algorithm,
    pub hyperparams: JsonB,
    pub status: String,
    pub metrics: Option<JsonB>,
    pub search: Option<Search>,
    pub search_params: JsonB,
    pub search_args: JsonB,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    estimator: Option<Box<dyn Estimator>>,
}

impl Model {
    pub fn create(
        project: &Project,
        snapshot: &Snapshot,
        algorithm: Algorithm,
        hyperparams: JsonB,
        search: Option<Search>,
        search_params: JsonB,
        search_args: JsonB,
    ) -> Model {
        let mut model: Option<Model> = None;

        Spi::connect(|client| {
            let result = client.select("
          INSERT INTO pgml_rust.models (project_id, snapshot_id, algorithm, hyperparams, status, search, search_params, search_args) 
          VALUES ($1, $2, $3, $4, $5, $6::pgml_rust.search, $7, $8) 
          RETURNING id, project_id, snapshot_id, algorithm, hyperparams, status, metrics, search, search_params, search_args, created_at, updated_at;",
              Some(1),
              Some(vec![
                  (PgBuiltInOids::INT8OID.oid(), project.id.into_datum()),
                  (PgBuiltInOids::INT8OID.oid(), snapshot.id.into_datum()),
                  (PgBuiltInOids::TEXTOID.oid(), algorithm.to_string().into_datum()),
                  (PgBuiltInOids::JSONBOID.oid(), hyperparams.into_datum()),
                  (PgBuiltInOids::TEXTOID.oid(), "new".to_string().into_datum()),
                  (PgBuiltInOids::TEXTOID.oid(), search.into_datum()),
                  (PgBuiltInOids::JSONBOID.oid(), search_params.into_datum()),
                  (PgBuiltInOids::JSONBOID.oid(), search_args.into_datum()),
              ])
          ).first();
            if result.len() > 0 {
                model = Some(Model {
                    id: result.get_datum(1).unwrap(),
                    project_id: result.get_datum(2).unwrap(),
                    snapshot_id: result.get_datum(3).unwrap(),
                    algorithm: Algorithm::from_str(result.get_datum(4).unwrap()).unwrap(),
                    hyperparams: result.get_datum(5).unwrap(),
                    status: result.get_datum(6).unwrap(),
                    metrics: result.get_datum(7),
                    search: search, // TODO
                    search_params: result.get_datum(9).unwrap(),
                    search_args: result.get_datum(10).unwrap(),
                    created_at: result.get_datum(11).unwrap(),
                    updated_at: result.get_datum(12).unwrap(),
                    estimator: None,
                });
            }

            Ok(Some(1))
        });
        let mut model = model.unwrap();
        let dataset = snapshot.dataset();
        model.fit(&project, &dataset);
        model.test(&project, &dataset);
        model
    }

    fn fit(&mut self, project: &Project, dataset: &Dataset) {
        let hyperparams: &serde_json::Value = &self.hyperparams.0;
        let hyperparams = hyperparams.as_object().unwrap();

        self.estimator = match self.algorithm {
            Algorithm::linear => {
                let x_train = Array2::from_shape_vec(
                    (dataset.num_train_rows, dataset.num_features),
                    dataset.x_train().to_vec(),
                )
                .unwrap();
                let y_train =
                    Array1::from_shape_vec(dataset.num_train_rows, dataset.y_train().to_vec())
                        .unwrap();
                let estimator: Option<Box<dyn Estimator>> = match project.task {
                    Task::regression => Some(Box::new(
                        smartcore::linear::linear_regression::LinearRegression::fit(
                            &x_train,
                            &y_train,
                            Default::default(),
                        )
                        .unwrap(),
                    )),
                    Task::classification => Some(Box::new(
                        smartcore::linear::logistic_regression::LogisticRegression::fit(
                            &x_train,
                            &y_train,
                            Default::default(),
                        )
                        .unwrap(),
                    )),
                };
                let bytes: Vec<u8> = rmp_serde::to_vec(&*estimator.as_ref().unwrap()).unwrap();
                Spi::get_one_with_args::<i64>(
                  "INSERT INTO pgml_rust.files (model_id, path, part, data) VALUES($1, 'estimator.rmp', 0, $2) RETURNING id",
                  vec![
                      (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
                      (PgBuiltInOids::BYTEAOID.oid(), bytes.into_datum()),
                  ]
              ).unwrap();
                estimator
            }
            Algorithm::xgboost => {
                let mut dtrain =
                    DMatrix::from_dense(&dataset.x_train(), dataset.num_train_rows).unwrap();
                let mut dtest =
                    DMatrix::from_dense(&dataset.x_test(), dataset.num_test_rows).unwrap();
                dtrain.set_labels(&dataset.y_train()).unwrap();
                dtest.set_labels(&dataset.y_test()).unwrap();

                // specify datasets to evaluate against during training
                let evaluation_sets = &[(&dtrain, "train"), (&dtest, "test")];

                // configure objectives, metrics, etc.
                let learning_params =
                    parameters::learning::LearningTaskParametersBuilder::default()
                        .objective(match project.task {
                            Task::regression => xgboost::parameters::learning::Objective::RegLinear,
                            Task::classification => {
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

                bst.save(std::path::Path::new(&path)).unwrap();

                let bytes = std::fs::read(&path).unwrap();
                Spi::get_one_with_args::<i64>(
                  "INSERT INTO pgml_rust.files (model_id, path, part, data) VALUES($1, 'estimator.rmp', 0, $2) RETURNING id",
                  vec![
                      (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
                      (PgBuiltInOids::BYTEAOID.oid(), bytes.into_datum()),
                  ]
            ).unwrap();
                Some(Box::new(BoosterBox::new(bst)))
            }
        };
    }

    fn test(&mut self, project: &Project, dataset: &Dataset) {
        let metrics = self
            .estimator
            .as_ref()
            .unwrap()
            .test(project.task, &dataset);
        self.metrics = Some(JsonB(json!(metrics.clone())));
        Spi::get_one_with_args::<i64>(
            "UPDATE pgml_rust.models SET metrics = $1 WHERE id = $2 RETURNING id",
            vec![
                (
                    PgBuiltInOids::JSONBOID.oid(),
                    JsonB(json!(metrics)).into_datum(),
                ),
                (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
            ],
        )
        .unwrap();
    }
}
