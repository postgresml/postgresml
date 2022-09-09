use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use ndarray::{Array1, Array2};
use pgx::*;
use once_cell::sync::Lazy;
use serde_json::json;

use crate::orm::Algorithm;
use crate::orm::Dataset;
use crate::orm::Estimator;
use crate::orm::Project;
use crate::orm::Search;
use crate::orm::Snapshot;
use crate::orm::Task;

static DEPLOYED_MODELS_BY_PROJECT_ID: Lazy<Mutex<HashMap<i64, Arc<Model>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Model")
    }
}

impl Model {
    pub fn find_deployed(project_id: i64) -> Option<Arc<Model>> {
        {
            let models = DEPLOYED_MODELS_BY_PROJECT_ID.lock().unwrap();
            let model = models.get(&project_id);
            if model.is_some() {
                info!("cache hit model: {}", project_id);
                return Some(model.unwrap().clone());
            } else {
                info!("cache miss model: {}", project_id);
            }
        }

        let mut model: Option<Arc<Model>> = None;
        Spi::connect(|client| {
            let result = client.select("
          SELECT id, project_id, snapshot_id, algorithm, hyperparams, status, metrics, search, search_params, search_args, created_at, updated_at
          FROM pgml_rust.models 
          JOIN pgml_rust.deployments 
            ON deployments.model_id = models.id
            AND deployments.project_id = $1
          ORDER by deployments.created_at DESC
          LIMIT 1;",
              Some(1),
              Some(vec![
                  (PgBuiltInOids::INT8OID.oid(), project_id.into_datum()),
              ])
          ).first();
            if result.len() > 0 {
                info!("db hit model: {}", project_id);
                let mut models = DEPLOYED_MODELS_BY_PROJECT_ID.lock().unwrap();
                models.insert(
                    project_id,
                    Arc::new(Model {
                        id: result.get_datum(1).unwrap(),
                        project_id: result.get_datum(2).unwrap(),
                        snapshot_id: result.get_datum(3).unwrap(),
                        algorithm: Algorithm::from_str(result.get_datum(4).unwrap()).unwrap(),
                        hyperparams: result.get_datum(5).unwrap(),
                        status: result.get_datum(6).unwrap(),
                        metrics: result.get_datum(7),
                        search: result.get_datum(8),
                        search_params: result.get_datum(9).unwrap(),
                        search_args: result.get_datum(10).unwrap(),
                        created_at: result.get_datum(11).unwrap(),
                        updated_at: result.get_datum(12).unwrap(),
                        estimator: None,
                    }),
                );
                model = Some(models.get(&project_id).unwrap().clone());
            }
            Ok(Some(1))
        });

        model
    }

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
                match project.task {
                    Task::regression => {
                        Some(Box::new(
                            smartcore::linear::linear_regression::LinearRegression::fit(
                                &x_train,
                                &y_train,
                                Default::default(),
                            )
                            .unwrap()
                        ))
                    }
                    Task::classification => {
                        Some(Box::new(
                            smartcore::linear::logistic_regression::LogisticRegression::fit(
                                &x_train,
                                &y_train,
                                Default::default(),
                            )
                            .unwrap()
                        ))
                    }
                }
            },
            Algorithm::xgboost => {
                todo!()
            }
        };

        let bytes = rmp_serde::to_vec(&*self.estimator.as_ref().unwrap()).unwrap();
        Spi::get_one_with_args::<i64>(
          "INSERT INTO pgml_rust.files (model_id, path, part, data) VALUES($1, 'estimator.rmp', 0, $2) RETURNING id",
          vec![
              (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
              (PgBuiltInOids::BYTEAOID.oid(), bytes.into_datum()),
          ]
      ).unwrap();
    }

    fn test(&mut self, project: &Project, dataset: &Dataset) {
        let metrics = self.estimator.as_ref().unwrap().test(project.task, &dataset);
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

    pub fn predict(&mut self, features: Vec<f32>) -> f32 {
        self.estimator().estimator_predict(features)
    }

    pub fn estimator(&self) -> Box<dyn Estimator> {
        todo!()
        // match self.estimator {
        //     Some(estimator) => estimator,
        //     None => {
        //         let task = self.project_task();
        //         let estimator_data = self.estimator_data();
        //         self.estimator = match task {
        //             Task::classification => todo!(),
        //             Task::regression => match self.algorithm {
        //                 Algorithm::linear => {
        //                     Some(Box::new(rmp_serde::from_read::<&Vec<u8>, smartcore::linear::linear_regression::LinearRegression<f32, Array2<f32>>>(&estimator_data).unwrap()))
        //                 }
        //                 Algorithm::xgboost => todo!(),
        //             },
        //         };

        //         self.estimator.unwrap()
        //     }
        // }
    }

    fn estimator_data(&self) -> Vec<u8> {
        Spi::get_one_with_args::<&[u8]>("SELECT data FROM pgml_rust.files WHERE model_id = $1",
            vec![(PgBuiltInOids::INT8OID.oid(), self.id.into_datum())],
        ).expect("Model `{}` has no saved estimator").to_vec()
    }

    fn project_task(&self) -> Task {
        Spi::get_one_with_args::<Task>("SELECT task FROM pgml_rust.projects WHERE id = $1",
            vec![(PgBuiltInOids::INT8OID.oid(), self.project_id.into_datum())],
        ).expect("Model `{}` has no associated project")
    }
}
