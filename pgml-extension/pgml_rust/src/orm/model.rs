use pgx::*;
use serde_json::json;

use crate::orm::Algorithm;
use crate::orm::Dataset;
use crate::orm::Estimator;
use crate::orm::Project;
use crate::orm::Runtime;
use crate::orm::Search;
use crate::orm::Snapshot;
use crate::orm::Status;

use crate::bindings::lightgbm::{lightgbm_save, lightgbm_train};
use crate::bindings::sklearn::{sklearn_save, sklearn_search, sklearn_train};
use crate::bindings::xgboost::{xgboost_save, xgboost_train};

#[derive(Debug)]
pub struct Model {
    pub id: i64,
    pub project_id: i64,
    pub snapshot_id: i64,
    pub algorithm: Algorithm,
    pub hyperparams: JsonB,
    pub runtime: Runtime,
    pub status: Status,
    pub metrics: Option<JsonB>,
    pub search: Option<Search>,
    pub search_params: JsonB,
    pub search_args: JsonB,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    estimator: Option<Box<dyn Estimator>>,
}

impl Model {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        project: &Project,
        snapshot: &Snapshot,
        algorithm: Algorithm,
        hyperparams: JsonB,
        search: Option<Search>,
        search_params: JsonB,
        search_args: JsonB,
        runtime: Option<Runtime>,
    ) -> Model {
        let mut model: Option<Model> = None;

        // Set the runtime to one we recommend, unless the user knows better.
        let runtime = match runtime {
            Some(runtime) => runtime,
            None => match algorithm {
                Algorithm::xgboost => Runtime::rust,
                Algorithm::lightgbm => Runtime::rust,
                _ => Runtime::python,
            },
        };

        let dataset = snapshot.dataset();
        let status = Status::in_progress;
        // Create the model record.
        Spi::connect(|client| {
            let result = client.select("
          INSERT INTO pgml.models (project_id, snapshot_id, algorithm, runtime, hyperparams, status, search, search_params, search_args, num_features) 
          VALUES ($1, $2, cast($3 AS pgml.algorithm), cast($4 AS pgml.runtime), $5, cast($6 as pgml.status), $7, $8, $9, $10) 
          RETURNING id, project_id, snapshot_id, algorithm, runtime, hyperparams, status, metrics, search, search_params, search_args, created_at, updated_at;",
              Some(1),
              Some(vec![
                  (PgBuiltInOids::INT8OID.oid(), project.id.into_datum()),
                  (PgBuiltInOids::INT8OID.oid(), snapshot.id.into_datum()),
                  (PgBuiltInOids::TEXTOID.oid(), algorithm.to_string().into_datum()),
                  (PgBuiltInOids::TEXTOID.oid(), runtime.to_string().into_datum()),
                  (PgBuiltInOids::JSONBOID.oid(), hyperparams.into_datum()),
                  (PgBuiltInOids::TEXTOID.oid(), status.to_string().into_datum()),
                  (PgBuiltInOids::TEXTOID.oid(), search.map(|search| search.to_string()).into_datum()),
                  (PgBuiltInOids::JSONBOID.oid(), search_params.into_datum()),
                  (PgBuiltInOids::JSONBOID.oid(), search_args.into_datum()),
                  (PgBuiltInOids::INT4OID.oid(), dataset.num_features.into_datum()),
              ])
          ).first();
            if !result.is_empty() {
                model = Some(Model {
                    id: result.get_datum(1).unwrap(),
                    project_id: result.get_datum(2).unwrap(),
                    snapshot_id: result.get_datum(3).unwrap(),
                    algorithm, // 4
                    runtime,   // 5
                    hyperparams: result.get_datum(6).unwrap(),
                    status, // 6,
                    metrics: result.get_datum(8),
                    search, // 9
                    search_params: result.get_datum(10).unwrap(),
                    search_args: result.get_datum(11).unwrap(),
                    created_at: result.get_datum(12).unwrap(),
                    updated_at: result.get_datum(13).unwrap(),
                    estimator: None,
                });
            }

            Ok(Some(1))
        });

        let mut model = model.unwrap();

        model.fit(project, &dataset);
        model.test(project, &dataset);

        Spi::connect(|client| {
            client.select(
                "UPDATE pgml.models SET status = $1::pgml.status WHERE id = $2",
                Some(1),
                Some(vec![
                    (
                        PgBuiltInOids::TEXTOID.oid(),
                        Status::successful.to_string().into_datum(),
                    ),
                    (PgBuiltInOids::INT8OID.oid(), model.id.into_datum()),
                ]),
            );

            Ok(Some(1))
        });

        model
    }

    #[allow(non_snake_case)]
    fn fit(&mut self, project: &Project, dataset: &Dataset) {
        // Get the hyperparameters.
        let hyperparams: &serde_json::Value = &self.hyperparams.0;
        let mut hyperparams = hyperparams.as_object().unwrap().clone();

        // Train the estimator. We are getting the estimator struct and
        // it's serialized form to save into the `models` table.
        let (estimator, bytes): (Box<dyn Estimator>, Vec<u8>) = match self.runtime {
            Runtime::python => {
                let estimator = match self.search {
                    Some(search) => {
                        let (estimator, chosen_hyperparams) = sklearn_search(
                            project.task,
                            self.algorithm,
                            search,
                            dataset,
                            &hyperparams,
                            self.search_params.0.as_object().unwrap(),
                        );

                        hyperparams.extend(chosen_hyperparams);

                        estimator
                    }

                    None => sklearn_train(project.task, self.algorithm, dataset, &hyperparams),
                };

                let bytes = sklearn_save(&estimator);

                (Box::new(estimator), bytes)
            }
            Runtime::rust => {
                match self.algorithm {
                    Algorithm::xgboost => {
                        let estimator = xgboost_train(project.task, dataset, &hyperparams);

                        let bytes = xgboost_save(&estimator);

                        (Box::new(estimator), bytes)
                    }

                    Algorithm::lightgbm => {
                        let estimator = lightgbm_train(project.task, dataset, &hyperparams);
                        let bytes = lightgbm_save(&estimator);

                        (Box::new(estimator), bytes)
                    }

                    _ => todo!(),
                    // Algorithm::smartcore => {
                    //     let estimator =
                    //         smartcore_train(project.task, self.algorithm, dataset, &hyperparams);

                    //     let bytes = smartcore_save(&estimator);

                    //     (estimator, bytes)
                    // },
                }
            }
        };

        // Save the estimator.
        Spi::get_one_with_args::<i64>(
          "INSERT INTO pgml.files (model_id, path, part, data) VALUES($1, 'estimator.rmp', 0, $2) RETURNING id",
          vec![
              (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
              (PgBuiltInOids::BYTEAOID.oid(), bytes.into_datum()),
          ]
        ).unwrap();

        // Save the hyperparams after search
        Spi::get_one_with_args::<i64>(
            "UPDATE pgml.models SET hyperparams = $1::jsonb WHERE id = $2 RETURNING id",
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
            "UPDATE pgml.models SET metrics = $1 WHERE id = $2 RETURNING id",
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
