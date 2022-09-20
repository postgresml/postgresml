use std::str::FromStr;

use pgx::*;
use serde_json::json;

use crate::engines::engine::Engine;
use crate::orm::Algorithm;
use crate::orm::Dataset;
use crate::orm::Estimator;
use crate::orm::Project;
use crate::orm::Search;
use crate::orm::Snapshot;

use crate::engines::sklearn::{sklearn_save, sklearn_search, sklearn_train};
use crate::engines::smartcore::{smartcore_save, smartcore_train};
use crate::engines::xgboost::{xgboost_save, xgboost_train};

#[derive(Debug)]
pub struct Model {
    pub id: i64,
    pub project_id: i64,
    pub snapshot_id: i64,
    pub algorithm: Algorithm,
    pub hyperparams: JsonB,
    pub engine: Engine,
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
        engine: Option<Engine>,
    ) -> Model {
        let mut model: Option<Model> = None;

        // Set the engine to one we recommend, unless the user knows better.
        let engine = match engine {
            Some(engine) => engine,
            None => match algorithm {
                Algorithm::xgboost => Engine::xgboost,
                Algorithm::linear => Engine::sklearn,
                Algorithm::svm => Engine::sklearn,
                Algorithm::lasso => Engine::sklearn,
                Algorithm::elastic_net => Engine::sklearn,
                Algorithm::ridge => Engine::sklearn,
                Algorithm::kmeans => Engine::sklearn,
                Algorithm::dbscan => Engine::sklearn,
                Algorithm::knn => Engine::sklearn,
                Algorithm::random_forest => Engine::sklearn,
            },
        };

        // Create the model record.
        Spi::connect(|client| {
            let result = client.select("
          INSERT INTO pgml_rust.models (project_id, snapshot_id, algorithm, hyperparams, status, search, search_params, search_args, engine) 
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
          RETURNING id, project_id, snapshot_id, algorithm, hyperparams, status, metrics, search, search_params, search_args, created_at, updated_at;",
              Some(1),
              Some(vec![
                  (PgBuiltInOids::INT8OID.oid(), project.id.into_datum()),
                  (PgBuiltInOids::INT8OID.oid(), snapshot.id.into_datum()),
                  (PgBuiltInOids::TEXTOID.oid(), algorithm.to_string().into_datum()),
                  (PgBuiltInOids::JSONBOID.oid(), hyperparams.into_datum()),
                  (PgBuiltInOids::TEXTOID.oid(), "new".to_string().into_datum()),
                  (PgBuiltInOids::TEXTOID.oid(), match search {
                    Some(search) => Some(search.to_string()),
                    None => None,
                  }.into_datum()),
                  (PgBuiltInOids::JSONBOID.oid(), search_params.into_datum()),
                  (PgBuiltInOids::JSONBOID.oid(), search_args.into_datum()),
                  (PgBuiltInOids::TEXTOID.oid(), engine.to_string().into_datum()),
              ])
          ).first();
            if !result.is_empty() {
                model = Some(Model {
                    id: result.get_datum(1).unwrap(),
                    project_id: result.get_datum(2).unwrap(),
                    snapshot_id: result.get_datum(3).unwrap(),
                    algorithm: Algorithm::from_str(result.get_datum(4).unwrap()).unwrap(),
                    hyperparams: result.get_datum(5).unwrap(),
                    status: result.get_datum(6).unwrap(),
                    metrics: result.get_datum(7),
                    search, // TODO
                    search_params: result.get_datum(9).unwrap(),
                    search_args: result.get_datum(10).unwrap(),
                    created_at: result.get_datum(11).unwrap(),
                    updated_at: result.get_datum(12).unwrap(),
                    engine,
                    estimator: None,
                });
            }

            Ok(Some(1))
        });

        let mut model = model.unwrap();
        let dataset = snapshot.dataset();

        model.fit(project, &dataset);
        model.test(project, &dataset);

        model
    }

    #[allow(non_snake_case)]
    fn fit(&mut self, project: &Project, dataset: &Dataset) {
        // Get the hyperparameters.
        let hyperparams: &serde_json::Value = &self.hyperparams.0;
        let mut hyperparams = hyperparams.as_object().unwrap().clone();

        // Train the estimator. We are getting the estimator struct and
        // it's serialized form to save into the `models` table.
        let (estimator, bytes): (Box<dyn Estimator>, Vec<u8>) = match self.engine {
            Engine::sklearn => {
                let estimator = match self.search {
                    Some(search) => {
                        let (estimator, chosen_hyperparams) = sklearn_search(
                            project.task,
                            self.algorithm,
                            search,
                            dataset,
                            &hyperparams,
                            &self.search_params.0.as_object().unwrap(),
                        );

                        hyperparams.extend(chosen_hyperparams);

                        estimator
                    }

                    None => sklearn_train(project.task, self.algorithm, dataset, &hyperparams),
                };

                let bytes = sklearn_save(&estimator);

                (Box::new(estimator), bytes)
            }

            Engine::xgboost => {
                let estimator = xgboost_train(project.task, dataset, &hyperparams);

                let bytes = xgboost_save(&estimator);

                (Box::new(estimator), bytes)
            }

            Engine::smartcore => {
                let estimator =
                    smartcore_train(project.task, self.algorithm, dataset, &hyperparams);

                let bytes = smartcore_save(&estimator);

                (estimator, bytes)
            }

            _ => todo!(),
        };

        // Save the estimator
        Spi::get_one_with_args::<i64>(
          "INSERT INTO pgml_rust.files (model_id, path, part, data) VALUES($1, 'estimator.rmp', 0, $2) RETURNING id",
          vec![
              (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
              (PgBuiltInOids::BYTEAOID.oid(), bytes.into_datum()),
          ]
        ).unwrap();

        Spi::get_one_with_args::<i64>(
            "UPDATE pgml_rust.models SET hyperparams = $1::jsonb WHERE id = $2 RETURNING id",
            vec![
                (
                    PgBuiltInOids::TEXTOID.oid(),
                    serde_json::to_string(&hyperparams).unwrap().into_datum(),
                ),
                (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
            ],
        )
        .unwrap();

        self.estimator = Some(estimator);
    }

    fn test(&mut self, project: &Project, dataset: &Dataset) {
        let metrics = self.estimator.as_ref().unwrap().test(project.task, dataset);
        self.metrics = Some(JsonB(json!(metrics)));
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
