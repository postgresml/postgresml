use ndarray::{Array, Array1, Array2};
use once_cell::sync::Lazy;
use pgx::*;
use serde::Deserialize;
use serde_json;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::string::ToString;
use std::sync::Arc;
use std::sync::Mutex;

static PROJECTS: Lazy<Mutex<HashMap<String, Arc<Project>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(PostgresEnum, Copy, Clone, PartialEq, Debug)]
#[allow(non_camel_case_types)]
enum Algorithm {
    linear,
    xgboost,
}

impl std::str::FromStr for Algorithm {
    type Err = ();

    fn from_str(input: &str) -> Result<Algorithm, Self::Err> {
        match input {
            "linear" => Ok(Algorithm::linear),
            "xgboost" => Ok(Algorithm::xgboost),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Algorithm {
    fn to_string(&self) -> String {
        match *self {
            Algorithm::linear => "linear".to_string(),
            Algorithm::xgboost => "xgboost".to_string(),
        }
    }
}

#[derive(PostgresEnum, Copy, Clone, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
enum Task {
    regression,
    classification,
}

impl std::str::FromStr for Task {
    type Err = ();

    fn from_str(input: &str) -> Result<Task, Self::Err> {
        match input {
            "regression" => Ok(Task::regression),
            "classification" => Ok(Task::classification),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Task {
    fn to_string(&self) -> String {
        match *self {
            Task::regression => "regression".to_string(),
            Task::classification => "classification".to_string(),
        }
    }
}

#[derive(PostgresEnum, Copy, Clone, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
enum Sampling {
    random,
    last,
}

impl std::str::FromStr for Sampling {
    type Err = ();

    fn from_str(input: &str) -> Result<Sampling, Self::Err> {
        match input {
            "random" => Ok(Sampling::random),
            "last" => Ok(Sampling::last),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Sampling {
    fn to_string(&self) -> String {
        match *self {
            Sampling::random => "random".to_string(),
            Sampling::last => "last".to_string(),
        }
    }
}

#[derive(PostgresEnum, Copy, Clone, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
enum Search {
    grid,
    random,
    none,
}

impl std::str::FromStr for Search {
    type Err = ();

    fn from_str(input: &str) -> Result<Search, Self::Err> {
        match input {
            "grid" => Ok(Search::grid),
            "random" => Ok(Search::random),
            "none" => Ok(Search::none),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Search {
    fn to_string(&self) -> String {
        match *self {
            Search::grid => "grid".to_string(),
            Search::random => "random".to_string(),
            Search::none => "none".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Project {
    id: i64,
    name: String,
    task: Task,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl Project {
    fn find(id: i64) -> Option<Project> {
        let mut project: Option<Project> = None;

        Spi::connect(|client| {
            let result = client.select("SELECT id, name, task, created_at, updated_at FROM pgml_rust.projects WHERE id = $1 LIMIT 1;",
                Some(1),
                Some(vec![
                    (PgBuiltInOids::INT8OID.oid(), id.into_datum()),
                ])
            ).first();
            if result.len() > 0 {
                project = Some(Project {
                    id: result.get_datum(1).unwrap(),
                    name: result.get_datum(2).unwrap(),
                    task: Task::from_str(result.get_datum(3).unwrap()).unwrap(),
                    created_at: result.get_datum(4).unwrap(),
                    updated_at: result.get_datum(5).unwrap(),
                });
            }
            Ok(Some(1))
        });

        project
    }

    fn find_by_name(name: &str) -> Option<Arc<Project>> {
        {
            let projects = PROJECTS.lock().unwrap();
            let project = projects.get(name);
            if project.is_some() {
                info!("cache hit: {}", name);
                return Some(project.unwrap().clone());
            } else {
                info!("cache miss: {}", name);
            }
        }

        let mut project = None;

        Spi::connect(|client| {
            let result = client.select("SELECT id, name, task, created_at, updated_at FROM pgml_rust.projects WHERE name = $1 LIMIT 1;",
                Some(1),
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), name.into_datum()),
                ])
            ).first();
            if result.len() > 0 {
                info!("db hit: {}", name);
                let mut projects = PROJECTS.lock().unwrap();
                projects.insert(
                    name.to_string(),
                    Arc::new(Project {
                        id: result.get_datum(1).unwrap(),
                        name: result.get_datum(2).unwrap(),
                        task: Task::from_str(result.get_datum(3).unwrap()).unwrap(),
                        created_at: result.get_datum(4).unwrap(),
                        updated_at: result.get_datum(5).unwrap(),
                    }),
                );
                project = Some(projects.get(name).unwrap().clone());
            } else {
                info!("db miss: {}", name);
            }
            Ok(Some(1))
        });

        project
    }

    fn create(name: &str, task: Task) -> Arc<Project> {
        let mut project: Option<Arc<Project>> = None;

        Spi::connect(|client| {
            let result = client.select("INSERT INTO pgml_rust.projects (name, task) VALUES ($1, $2) RETURNING id, name, task, created_at, updated_at;",
                Some(1),
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), name.into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), task.to_string().into_datum()),
                ])
            ).first();
            if result.len() > 0 {
                let mut projects = PROJECTS.lock().unwrap();
                projects.insert(
                    name.to_string(),
                    Arc::new(Project {
                        id: result.get_datum(1).unwrap(),
                        name: result.get_datum(2).unwrap(),
                        task: Task::from_str(result.get_datum(3).unwrap()).unwrap(),
                        created_at: result.get_datum(4).unwrap(),
                        updated_at: result.get_datum(5).unwrap(),
                    }),
                );
                project = Some(projects.get(name).unwrap().clone());
            }
            Ok(Some(1))
        });
        info!("create project: {:?}", project.as_ref().unwrap());
        project.unwrap()
    }

    fn last_snapshot(&self) -> Option<Snapshot> {
        Snapshot::find_last_by_project_id(self.id)
    }
}

pub struct Data {
    x: Vec<f32>,
    y: Vec<f32>,
    num_features: usize,
    num_labels: usize,
    num_rows: usize,
    num_train_rows: usize,
    num_test_rows: usize,
}

impl Data {
    fn x_train(&self) -> &[f32] {
        &self.x[..self.num_train_rows * self.num_features]
    }

    fn x_test(&self) -> &[f32] {
        &self.x[self.num_train_rows * self.num_features..]
    }

    fn y_train(&self) -> &[f32] {
        &self.y[..self.num_train_rows * self.num_labels]
    }

    fn y_test(&self) -> &[f32] {
        &self.y[self.num_train_rows * self.num_labels..]
    }
}

#[derive(Debug)]
pub struct Snapshot {
    id: i64,
    relation_name: String,
    y_column_name: Vec<String>,
    test_size: f32,
    test_sampling: Sampling,
    status: String,
    columns: Option<JsonB>,
    analysis: Option<JsonB>,
    created_at: Timestamp,
    updated_at: Timestamp,
}

impl Snapshot {
    fn find_last_by_project_id(project_id: i64) -> Option<Snapshot> {
        let mut snapshot = None;
        Spi::connect(|client| {
            let result = client.select(
                    "SELECT snapshots.id, snapshots.relation_name, snapshots.y_column_name, snapshots.test_size, snapshots.test_sampling, snapshots.status, snapshots.columns, snapshots.analysis, snapshots.created_at, snapshots.updated_at 
                    FROM pgml_rust.snapshots 
                    JOIN pgml_rust.models
                      ON models.snapshot_id = snapshots.id
                      AND models.project_id = $1 
                    ORDER BY snapshots.id DESC 
                    LIMIT 1;
                    ",
                Some(1),
                Some(vec![
                    (PgBuiltInOids::INT8OID.oid(), project_id.into_datum()),
                ])
            ).first();
            if result.len() > 0 {
                snapshot = Some(Snapshot {
                    id: result.get_datum(1).unwrap(),
                    relation_name: result.get_datum(2).unwrap(),
                    y_column_name: result.get_datum(3).unwrap(),
                    test_size: result.get_datum(4).unwrap(),
                    test_sampling: Sampling::from_str(result.get_datum(5).unwrap()).unwrap(),
                    status: result.get_datum(6).unwrap(),
                    columns: result.get_datum(7),
                    analysis: result.get_datum(8),
                    created_at: result.get_datum(9).unwrap(),
                    updated_at: result.get_datum(10).unwrap(),
                });
            }
            Ok(Some(1))
        });
        snapshot
    }

    fn create(
        relation_name: &str,
        y_column_name: &str,
        test_size: f32,
        test_sampling: Sampling,
    ) -> Snapshot {
        let mut snapshot: Option<Snapshot> = None;

        Spi::connect(|client| {
            let result = client.select("INSERT INTO pgml_rust.snapshots (relation_name, y_column_name, test_size, test_sampling, status) VALUES ($1, $2, $3, $4, $5) RETURNING id, relation_name, y_column_name, test_size, test_sampling, status, columns, analysis, created_at, updated_at;",
                Some(1),
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), relation_name.into_datum()),
                    (PgBuiltInOids::TEXTARRAYOID.oid(), vec![y_column_name].into_datum()),
                    (PgBuiltInOids::FLOAT4OID.oid(), test_size.into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), test_sampling.to_string().into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), "new".to_string().into_datum()),
                ])
            ).first();
            let mut s = Snapshot {
                id: result.get_datum(1).unwrap(),
                relation_name: result.get_datum(2).unwrap(),
                y_column_name: result.get_datum(3).unwrap(),
                test_size: result.get_datum(4).unwrap(),
                test_sampling: Sampling::from_str(result.get_datum(5).unwrap()).unwrap(),
                status: result.get_datum(6).unwrap(),
                columns: None,
                analysis: None,
                created_at: result.get_datum(9).unwrap(),
                updated_at: result.get_datum(10).unwrap(),
            };
            let mut sql = format!(
                r#"CREATE TABLE "pgml_rust"."snapshot_{}" AS SELECT * FROM {}"#,
                s.id, s.relation_name
            );
            if s.test_sampling == Sampling::random {
                sql += " ORDER BY random()";
            }
            client.select(&sql, None, None);
            client.select(
                r#"UPDATE "pgml_rust"."snapshots" SET status = 'snapped' WHERE id = $1"#,
                None,
                Some(vec![(PgBuiltInOids::INT8OID.oid(), s.id.into_datum())]),
            );
            s.analyze();
            snapshot = Some(s);
            Ok(Some(1))
        });

        snapshot.unwrap()
    }

    fn analyze(&mut self) {
        Spi::connect(|client| {
            let parts = self
                .relation_name
                .split(".")
                .map(|name| name.to_string())
                .collect::<Vec<String>>();
            let (schema_name, table_name) = match parts.len() {
                1 => (String::from("public"), parts[0].clone()),
                2 => (parts[0].clone(), parts[1].clone()),
                _ => error!(
                    "Relation name {} is not parsable into schema name and table name",
                    self.relation_name
                ),
            };
            let mut columns = HashMap::<String, String>::new();
            client.select("SELECT column_name::TEXT, data_type::TEXT FROM information_schema.columns WHERE table_schema = $1 AND table_name = $2",
                None,
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), schema_name.into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), table_name.into_datum()),
                ]))
            .for_each(|row| {
                columns.insert(row[1].value::<String>().unwrap(), row[2].value::<String>().unwrap());
            });

            for column in &self.y_column_name {
                if !columns.contains_key(column) {
                    error!(
                        "Column `{}` not found. Did you pass the correct `y_column_name`?",
                        column
                    )
                }
            }

            // We have to pull this analysis data into Rust as opposed to using Postgres
            // json_build_object(...), because Postgres functions have a limit of 100 arguments.
            // Any table that has more than 10 columns will exceed the Postgres limit since we
            // calculate 10 statistics per column.
            let mut stats = vec![r#"count(*)::FLOAT4 AS "samples""#.to_string()];
            let mut fields = vec!["samples".to_string()];
            for (column, data_type) in &columns {
                match data_type.as_str() {
                    "real" | "double precision" | "smallint" | "integer" | "bigint" | "boolean" => {
                        let column = column.to_string();
                        let quoted_column = match data_type.as_str() {
                            "boolean" => format!(r#""{}"::INT"#, column),
                            _ => format!(r#""{}""#, column),
                        };
                        stats.push(format!(r#"min({quoted_column})::FLOAT4 AS "{column}_min""#));
                        stats.push(format!(r#"max({quoted_column})::FLOAT4 AS "{column}_max""#));
                        stats.push(format!(
                            r#"avg({quoted_column})::FLOAT4 AS "{column}_mean""#
                        ));
                        stats.push(format!(
                            r#"stddev({quoted_column})::FLOAT4 AS "{column}_stddev""#
                        ));
                        stats.push(format!(r#"percentile_disc(0.25) within group (order by {quoted_column})::FLOAT4 AS "{column}_p25""#));
                        stats.push(format!(r#"percentile_disc(0.5) within group (order by {quoted_column})::FLOAT4 AS "{column}_p50""#));
                        stats.push(format!(r#"percentile_disc(0.75) within group (order by {quoted_column})::FLOAT4 AS "{column}_p75""#));
                        stats.push(format!(
                            r#"count({quoted_column})::FLOAT4 AS "{column}_count""#
                        ));
                        stats.push(format!(
                            r#"count(distinct {quoted_column})::FLOAT4 AS "{column}_distinct""#
                        ));
                        stats.push(format!(
                            r#"sum(({quoted_column} IS NULL)::INT)::FLOAT4 AS "{column}_nulls""#
                        ));
                        fields.push(format!("{column}_min"));
                        fields.push(format!("{column}_max"));
                        fields.push(format!("{column}_mean"));
                        fields.push(format!("{column}_stddev"));
                        fields.push(format!("{column}_p25"));
                        fields.push(format!("{column}_p50"));
                        fields.push(format!("{column}_p75"));
                        fields.push(format!("{column}_count"));
                        fields.push(format!("{column}_distinct"));
                        fields.push(format!("{column}_nulls"));
                    }
                    &_ => {}
                }
            }

            let stats = stats.join(",");
            let sql = format!(r#"SELECT {stats} FROM "pgml_rust"."snapshot_{}""#, self.id);
            let result = client.select(&sql, Some(1), None).first();
            let mut analysis = HashMap::new();
            for (i, field) in fields.iter().enumerate() {
                analysis.insert(
                    field.to_owned(),
                    result
                        .get_datum::<f32>((i + 1).try_into().unwrap())
                        .unwrap(),
                );
            }
            let analysis_datum = JsonB(json!(analysis.clone()));
            let column_datum = JsonB(json!(columns.clone()));
            self.analysis = Some(JsonB(json!(analysis)));
            self.columns = Some(JsonB(json!(columns)));
            client.select("UPDATE pgml_rust.snapshots SET status = 'complete', analysis = $1, columns = $2 WHERE id = $3", Some(1), Some(vec![
                (PgBuiltInOids::JSONBOID.oid(), analysis_datum.into_datum()),
                (PgBuiltInOids::JSONBOID.oid(), column_datum.into_datum()),
                (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
            ]));

            Ok(Some(1))
        });
    }

    fn data(&self) -> Data {
        let mut data = None;
        Spi::connect(|client| {
            let json: &serde_json::Value = &self.columns.as_ref().unwrap().0;
            let feature_columns = json
                .as_object()
                .unwrap()
                .keys()
                .filter_map(|column| match self.y_column_name.contains(column) {
                    true => None,
                    false => Some(format!("{}::FLOAT4", column)),
                })
                .collect::<Vec<String>>();
            let label_columns = self
                .y_column_name
                .iter()
                .map(|column| format!("{}::FLOAT4", column))
                .collect::<Vec<String>>();

            let sql = format!(
                "SELECT {}, {} FROM {}",
                feature_columns.join(", "),
                label_columns.join(", "),
                self.snapshot_name()
            );

            info!("Fetching data: {}", sql);
            let result = client.select(&sql, None, None);
            let mut x = Vec::with_capacity(result.len() * feature_columns.len());
            let mut y = Vec::with_capacity(result.len() * label_columns.len());
            result.for_each(|row| {
                // Postgres Arrays arrays are 1 indexed and so are SPI tuples...
                for i in 1..feature_columns.len() + 1 {
                    x.push(row[i].value::<f32>().unwrap());
                }
                for j in feature_columns.len() + 1..feature_columns.len() + label_columns.len() + 1
                {
                    y.push(row[j].value::<f32>().unwrap());
                }
            });
            let num_rows = x.len() / feature_columns.len();
            let num_test_rows = if self.test_size > 1.0 {
                self.test_size as usize
            } else {
                (num_rows as f32 * self.test_size).round() as usize
            };
            let num_train_rows = num_rows - num_test_rows;
            if num_train_rows <= 0 {
                error!(
                    "test_size = {} is too large. There are only {} samples.",
                    num_test_rows, num_rows
                );
            }
            info!(
                "got features {:?} labels {:?} rows {:?}",
                feature_columns.len(),
                label_columns.len(),
                num_rows
            );
            data = Some(Data {
                x: x,
                y: y,
                num_features: feature_columns.len(),
                num_labels: label_columns.len(),
                num_rows: num_rows,
                num_test_rows: num_test_rows,
                num_train_rows: num_train_rows,
            });

            Ok(Some(()))
        });

        data.unwrap()
    }

    fn snapshot_name(&self) -> String {
        format!("pgml_rust.snapshot_{}", self.id)
    }
}

// struct Estimator {
//     estimator: Box<dyn Estimation>,
// }

// impl Estimator {
//     fn test(&self, data: &Data) -> HashMap<String, f32> {
//         self.estimator.test(data)
//     }
// }

// impl fmt::Debug for Estimator {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Estimator")
//     }
// }

trait Estimator {
    fn test(&self, data: &Data) -> HashMap<String, f32>;
    // fn predict();
    // fn predict_batch();
    // fn serialize();
}

impl Estimator for dyn smartcore::api::Predictor<Array2<f32>, Array1<f32>> {
    fn test(&self, data: &Data) -> HashMap<std::string::String, f32> {
        let x_test = Array2::from_shape_vec(
            (data.num_test_rows, data.num_features),
            data.x_test().to_vec(),
        )
        .unwrap();
        let y_hat = self.predict(&x_test).unwrap();
        let mut results = HashMap::new();
        if data.num_labels == 1 {
            let y_test = Array1::from_shape_vec(data.num_test_rows, data.y_test().to_vec()).unwrap();
            results.insert("r2".to_string(), smartcore::metrics::r2(&y_test, &y_hat));
            results.insert(
                "mse".to_string(),
                smartcore::metrics::mean_squared_error(&y_test, &y_hat),
            );
        }
        results
    }
}



struct Model<'a> {
    id: i64,
    project_id: i64,
    snapshot_id: i64,
    algorithm: Algorithm,
    hyperparams: JsonB,
    status: String,
    metrics: Option<JsonB>,
    search: Option<Search>,
    search_params: JsonB,
    search_args: JsonB,
    created_at: Timestamp,
    updated_at: Timestamp,
    project: Option<&'a Project>,
    snapshot: Option<&'a Snapshot>,
    estimator: Option<Box<dyn smartcore::api::Predictor<Array2<f32>, Array1<f32>>>>,
}

impl fmt::Debug for Model<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Model")
    }
}

impl Model<'_> {
    fn create<'a>(
        project: &'a Project,
        snapshot: &'a Snapshot,
        algorithm: Algorithm,
        hyperparams: JsonB,
        search: Option<Search>,
        search_params: JsonB,
        search_args: JsonB,
    ) -> Model<'a> {
        let mut model: Option<Model> = None;

        Spi::connect(|client| {
            let result = client.select("
            INSERT INTO pgml_rust.models (project_id, snapshot_id, algorithm, hyperparams, status, search, search_params, search_args) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
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
            let mut m = Model {
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
                project: Some(project),
                snapshot: Some(snapshot),
                estimator: None,
            };
            model = Some(m);
            Ok(Some(1))
        });
        let mut model = model.unwrap();
        model.fit();
        model
    }

    fn fit(&mut self) {
        info!("fitting model: {:?}", self.algorithm);
        let data = self.snapshot.unwrap().data();
        match self.algorithm {
            Algorithm::linear => {
                let x_train = Array2::from_shape_vec(
                    (data.num_train_rows, data.num_features),
                    data.x_train().to_vec(),
                )
                .unwrap();
                let y_train =
                    Array1::from_shape_vec(data.num_train_rows, data.y_train().to_vec()).unwrap();
                match self.project.unwrap().task {
                    Task::regression => {
                        self.estimator = Some(Box::new(
                            smartcore::linear::linear_regression::LinearRegression::fit(
                                &x_train,
                                &y_train,
                                Default::default(),
                            )
                            .unwrap(),
                        ))


                    }
                    Task::classification => {
                        self.estimator = Some(Box::new(
                            smartcore::linear::logistic_regression::LogisticRegression::fit(
                                &x_train,
                                &y_train,
                                Default::default(),
                            )
                            .unwrap(),
                        ))
                    }
                }

                let estimator = self.estimator.as_ref().unwrap();
                self.metrics = Some(JsonB(json!(estimator.test(&data))));
            },
            Algorithm::xgboost => {
                todo!()
            }
        }
        info!("fitting complete: {:?}", self.metrics);
    }

    // fn save<
    //     E: serde::Serialize + smartcore::api::Predictor<X, Y> + std::fmt::Debug,
    //     N: smartcore::math::num::RealNumber,
    //     X,
    //     Y: std::fmt::Debug + smartcore::linalg::BaseVector<N>,
    // >(
    //     estimator: E,
    //     x_test: X,
    //     y_test: Y,
    //     algorithm: OldAlgorithm,
    //     project_id: i64,
    // ) -> i64 {
    //     let y_hat = estimator.predict(&x_test).unwrap();

    //     info!("r2: {:?}", smartcore::metrics::r2(&y_test, &y_hat));
    //     info!(
    //         "mean squared error: {:?}",
    //         smartcore::metrics::mean_squared_error(&y_test, &y_hat)
    //     );

    //     let mut buffer = Vec::new();
    //     estimator
    //         .serialize(&mut Serializer::new(&mut buffer))
    //         .unwrap();
    //     info!("bin {:?}", buffer);

    //     let model_id = Spi::get_one_with_args::<i64>(
    //         "INSERT INTO pgml_rust.models (id, project_id, algorithm, data) VALUES (DEFAULT, $1, $2, $3) RETURNING id",
    //         vec![
    //             (PgBuiltInOids::INT8OID.oid(), project_id.into_datum()),
    //             (PgBuiltInOids::INT8OID.oid(), algorithm.into_datum()),
    //             (PgBuiltInOids::BYTEAOID.oid(), buffer.into_datum())
    //         ]
    //     ).unwrap();

    //     Spi::get_one_with_args::<i64>(
    //         "INSERT INTO pgml_rust.deployments (project_id, model_id, strategy) VALUES ($1, $2, 'last_trained') RETURNING id",
    //         vec![
    //             (PgBuiltInOids::INT8OID.oid(), project_id.into_datum()),
    //             (PgBuiltInOids::INT8OID.oid(), model_id.into_datum()),
    //         ]
    //     );
    //     model_id
    // }
}

#[pg_extern]
fn train(
    project_name: &str,
    task: Option<default!(Task, "NULL")>,
    relation_name: Option<default!(&str, "NULL")>,
    y_column_name: Option<default!(&str, "NULL")>,
    algorithm: default!(Algorithm, "'linear'"),
    hyperparams: default!(JsonB, "'{}'"),
    search: Option<default!(Search, "NULL")>,
    search_params: default!(JsonB, "'{}'"),
    search_args: default!(JsonB, "'{}'"),
    test_size: default!(f32, 0.25),
    test_sampling: default!(Sampling, "'last'"),
) {
    let project = match Project::find_by_name(project_name) {
        Some(project) => project,
        None => Project::create(project_name, task.unwrap()),
    };
    if task.is_some() && task.unwrap() != project.task {
        error!("Project `{:?}` already exists with a different task: `{:?}`. Create a new project instead.", project.name, project.task);
    }
    let snapshot = match relation_name {
        None => project.last_snapshot().expect("You must pass a `relation_name` and `y_column_name` to snapshot the first time you train a model."),
        Some(relation_name) => Snapshot::create(relation_name, y_column_name.expect("You must pass a `y_column_name` when you pass a `relation_name`"), test_size, test_sampling)
    };

    // # Default repeatable random state when possible
    // let algorithm = Model.algorithm_from_name_and_task(algorithm, task);
    // if "random_state" in algorithm().get_params() and "random_state" not in hyperparams:
    //     hyperparams["random_state"] = 0

    let model = Model::create(
        &project,
        &snapshot,
        algorithm,
        hyperparams,
        search,
        search_params,
        search_args,
    );

    info!("{:?}", project);
    info!("{:?}", snapshot);
    info!("{:?}", model);
}

// #[pg_extern]
// fn return_table_example() -> impl std::Iterator<Item = (name!(id, Option<i64>), name!(title, Option<String>))> {
//     let tuple = Spi::get_two_with_args("SELECT 1 AS id, 2 AS title;", None, None)
//     vec![tuple].into_iter()
// }

#[pg_extern]
fn create_snapshot(
    relation_name: &str,
    y_column_name: &str,
    test_size: f32,
    test_sampling: Sampling,
) -> i64 {
    let snapshot = Snapshot::create(relation_name, y_column_name, test_size, test_sampling);
    info!("{:?}", snapshot);
    snapshot.id
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn test_project_lifecycle() {
        assert_eq!(Project::create("test", Task::regression).id, 1);
        assert_eq!(Project::find(1).id, 1);
        assert_eq!(Project::find_by_name("test").name, "test");
    }

    #[pg_test]
    fn test_snapshot_lifecycle() {
        let snapshot = Snapshot::create("test", "column", 0.5, Sampling::last);
        assert_eq!(snapshot.id, 1);
    }
}
