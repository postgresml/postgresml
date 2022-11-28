use std::cmp::Ordering;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;
use std::collections::HashMap;

use ndarray::Zip;
use indexmap::IndexMap;
use pgx::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ndarray_stats::QuantileExt;
use pgx_pg_sys::stat;

use crate::orm::Dataset;
use crate::orm::Sampling;
use crate::orm::Status;


#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct Category {
    pub(crate) value: f32,
    pub(crate) members: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct Statistics {
    min: f32,
    max: f32,
    mean: f32,
    variance: f32,
    median: f32,
    mode: f32,
    missing: usize,
    distinct: usize,
    histogram: Vec<usize>,
    ventiles: Vec<f32>,
    categories: HashMap<String, Category>,
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics {
            min: f32::NAN,
            max: f32::NAN,
            mean: f32::NAN,
            variance: f32::NAN,
            median: f32::NAN,
            mode: f32::NAN,
            missing: 0,
            distinct: 0,
            histogram: vec![0; 20],
            ventiles: vec![f32::NAN; 19],
            categories: HashMap::new(),
        }
    }
}
// How to encode text values
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
pub(crate) enum Encode {
    // Encode each category as a unique integer value, this is a no-op for integer columns
    #[default]
    label,
    // Encode each category as one boolean column per category
    one_hot {
        #[serde(default)]
        limit: usize,
        #[serde(default)]
        min_frequency: f32
    },
    // Encode each category as ascending integer values
    ordinal(Vec<String>),
    // Encode each category as the mean of the target
    target(HashMap<String,f32>),
}

// How to replace missing values
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
pub(crate) enum Impute {
    #[default]
    mean,
    median,
    mode,
    min,
    max,
    // Replaces with 0
    zero,
    // Raises an error at runtime
    error,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
pub(crate) enum Scale {
    #[default]
    standard,
    min_max,
    max_abs,
    robust,
    preserve,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct Preprocessor {
    #[serde(default)]
    encode: Encode,
    #[serde(default)]
    impute: Impute,
    #[serde(default)]
    scale: Scale,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct Column {
    name: String,
    pg_type: String,
    nullable: bool,
    label: bool,
    position: usize,
    size: usize,
    preprocessor: Preprocessor,
    pub(crate) statistics: Statistics,
}

impl Column {
    fn quoted_name(&self) -> String {
        format!(r#""{}""#, self.name)
    }

    fn is_categorical(&self) -> bool {
        match self.pg_type.as_str() {
            "bool" | "bpchar" | "int2" | "int4" | "int8" | "text" | "varchar" => true,
            _ => false,
        }
    }
}

impl PartialOrd<Self> for Column {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Column {}

impl Ord for Column {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position.cmp(&other.position)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Snapshot {
    pub(crate) id: i64,
    pub(crate) relation_name: String,
    pub(crate) y_column_name: Vec<String>,
    pub(crate) test_size: f32,
    pub(crate) test_sampling: Sampling,
    pub(crate) status: Status,
    pub(crate) columns: Vec<Column>,
    pub(crate) analysis: Option<IndexMap<String, f32>>,
    pub(crate) created_at: Timestamp,
    pub(crate) updated_at: Timestamp,
    pub(crate) materialized: bool,
}

impl Display for Snapshot {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Snapshot {{ id: {}, relation_name: {}, y_column_name: {:?}, test_size: {}, test_sampling: {:?}, status: {:?} }}", self.id, self.relation_name, self.y_column_name, self.test_size, self.test_sampling, self.status)
    }
}

impl Snapshot {
    pub fn find(id: i64) -> Option<Snapshot> {
        let mut snapshot = None;
        Spi::connect(|client| {
            let result = client
                .select(
                    "SELECT
                        snapshots.id,
                        snapshots.relation_name,
                        snapshots.y_column_name,
                        snapshots.test_size,
                        snapshots.test_sampling::TEXT,
                        snapshots.status::TEXT,
                        snapshots.columns,
                        snapshots.analysis,
                        snapshots.created_at,
                        snapshots.updated_at,
                        snapshots.materialized
                    FROM pgml.snapshots 
                    WHERE id = $1 
                    ORDER BY snapshots.id DESC 
                    LIMIT 1;
                    ",
                    Some(1),
                    Some(vec![(PgBuiltInOids::INT8OID.oid(), id.into_datum())]),
                )
                .first();
            if !result.is_empty() {
                let jsonb: JsonB = result.get_datum(7).unwrap();
                let columns: Vec<Column> = serde_json::from_value(jsonb.0).unwrap();
                // let jsonb: JsonB = result.get_datum(8).unwrap();
                // let analysis: Option<IndexMap<String, f32>> = Some(serde_json::from_value(jsonb.0).unwrap());
                snapshot = Some(Snapshot {
                    id: result.get_datum(1).unwrap(),
                    relation_name: result.get_datum(2).unwrap(),
                    y_column_name: result.get_datum(3).unwrap(),
                    test_size: result.get_datum(4).unwrap(),
                    test_sampling: Sampling::from_str(result.get_datum(5).unwrap()).unwrap(),
                    status: Status::from_str(result.get_datum(6).unwrap()).unwrap(),
                    columns,
                    analysis: None,
                    created_at: result.get_datum(9).unwrap(),
                    updated_at: result.get_datum(10).unwrap(),
                    materialized: result.get_datum(11).unwrap(),
                });
            }
            Ok(Some(1))
        });
        snapshot
    }

    pub fn find_last_by_project_id(project_id: i64) -> Option<Snapshot> {
        let mut snapshot = None;
        Spi::connect(|client| {
            let result = client
                .select(
                    "SELECT
                        snapshots.id,
                        snapshots.relation_name,
                        snapshots.y_column_name,
                        snapshots.test_size,
                        snapshots.test_sampling::TEXT,
                        snapshots.status::TEXT,
                        snapshots.columns,
                        snapshots.analysis,
                        snapshots.created_at,
                        snapshots.updated_at,
                        snapshots.materialized
                    FROM pgml.snapshots 
                    JOIN pgml.models
                      ON models.snapshot_id = snapshots.id
                      AND models.project_id = $1 
                    ORDER BY snapshots.id DESC 
                    LIMIT 1;
                    ",
                    Some(1),
                    Some(vec![(
                        PgBuiltInOids::INT8OID.oid(),
                        project_id.into_datum(),
                    )]),
                )
                .first();
            if !result.is_empty() {
                let jsonb: JsonB = result.get_datum(7).unwrap();
                let columns: Vec<Column> = serde_json::from_value(jsonb.0).unwrap();
                let jsonb: JsonB = result.get_datum(8).unwrap();
                let analysis: Option<IndexMap<String, f32>> = Some(serde_json::from_value(jsonb.0).unwrap());
                snapshot = Some(Snapshot {
                    id: result.get_datum(1).unwrap(),
                    relation_name: result.get_datum(2).unwrap(),
                    y_column_name: result.get_datum(3).unwrap(),
                    test_size: result.get_datum(4).unwrap(),
                    test_sampling: Sampling::from_str(result.get_datum(5).unwrap()).unwrap(),
                    status: Status::from_str(result.get_datum(6).unwrap()).unwrap(),
                    columns,
                    analysis,
                    created_at: result.get_datum(9).unwrap(),
                    updated_at: result.get_datum(10).unwrap(),
                    materialized: result.get_datum(11).unwrap(),
                });
            }
            Ok(Some(1))
        });
        snapshot
    }

    pub fn create(
        relation_name: &str,
        y_column_name: Vec<String>,
        test_size: f32,
        test_sampling: Sampling,
        materialized: bool,
        preprocess: JsonB,
    ) -> Snapshot {
        let mut snapshot: Option<Snapshot> = None;
        let status = Status::in_progress;

        // Validate table exists.
        let (schema_name, table_name) = Self::fully_qualified_table(relation_name);

        let preprocessors: HashMap<String, Preprocessor> = serde_json::from_value(preprocess.0).expect("is valid");

        Spi::connect(|client| {
            let mut columns: Vec<Column> = Vec::new();
            client.select("SELECT column_name::TEXT, udt_name::TEXT, is_nullable::BOOLEAN, ordinal_position::INTEGER FROM information_schema.columns WHERE table_schema = $1 AND table_name = $2 ORDER BY ordinal_position ASC",
                None,
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), schema_name.into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), table_name.into_datum()),
                ]))
            .for_each(|row| {
                let name = row[1].value::<String>().unwrap();
                let mut pg_type = row[2].value::<String>().unwrap();
                let mut size = 1;
                if pg_type.starts_with('_') {
                    size = 0;
                    pg_type = pg_type[1..].to_string() + "[]";
                }
                let nullable = row[3].value::<bool>().unwrap();
                let position = row[4].value::<i32>().unwrap() as usize;
                let label = y_column_name.contains(&name);
                let statistics = Statistics::default();
                let preprocessor = match preprocessors.get(&name) {
                    Some(preprocessor) => preprocessor.clone(),
                    None => Preprocessor {
                        encode: Encode::label,
                        impute: Impute::mean,
                        scale: Scale::standard,
                    },
                };

                columns.push(
                    Column {
                        name,
                        pg_type,
                        nullable,
                        label,
                        position,
                        size,
                        statistics,
                        preprocessor,
                    }
                );
            });
            for column in &y_column_name {
                if !columns.iter().any(|c| c.label && &c.name == column) {
                    error!(
                        "Column `{}` not found. Did you pass the correct `y_column_name`?",
                        column
                    )
                }
            }

            let result = client.select("INSERT INTO pgml.snapshots (relation_name, y_column_name, test_size, test_sampling, status, columns, materialized) VALUES ($1, $2, $3, $4::pgml.sampling, $5::pgml.status, $6, $7) RETURNING id, relation_name, y_column_name, test_size, test_sampling::TEXT, status::TEXT, columns, analysis, created_at, updated_at;",
                Some(1),
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), relation_name.into_datum()),
                    (PgBuiltInOids::TEXTARRAYOID.oid(), y_column_name.into_datum()),
                    (PgBuiltInOids::FLOAT4OID.oid(), test_size.into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), test_sampling.to_string().into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), status.to_string().into_datum()),
                    (PgBuiltInOids::JSONBOID.oid(), JsonB(json!(columns)).into_datum()),
                    (PgBuiltInOids::BOOLOID.oid(), materialized.into_datum()),
                ])
            ).first();

            let mut s = Snapshot {
                id: result.get_datum(1).unwrap(),
                relation_name: result.get_datum(2).unwrap(),
                y_column_name: result.get_datum(3).unwrap(),
                test_size: result.get_datum(4).unwrap(),
                test_sampling: Sampling::from_str(result.get_datum(5).unwrap()).unwrap(),
                status,         // 6
                columns,        // 7
                analysis: None, // 8
                created_at: result.get_datum(9).unwrap(),
                updated_at: result.get_datum(10).unwrap(),
                materialized,
            };
            if materialized {
                let mut sql = format!(
                    r#"CREATE TABLE "pgml"."snapshot_{}" AS SELECT * FROM {}"#,
                    s.id, s.relation_name
                );
                if s.test_sampling == Sampling::random {
                    sql += " ORDER BY random()";
                }
                client.select(&sql, None, None);
            }
            snapshot = Some(s);
            Ok(Some(1))
        });

        snapshot.unwrap()
    }

    pub fn num_labels(&self) -> usize {
        self.y_column_name.len()
    }

    pub fn num_features(&self) -> usize {
        // TODO fix up for one hot encoding
        let mut num_features: usize = 0;
        for column in &self.columns {
            if !column.label {
                num_features += column.size;
            }
        }
        num_features
    }

    pub fn num_classes(&self) -> usize {
        let target = &self.y_column_name[0];
        *self.analysis.as_ref().unwrap()
            .get(&format!("{}_distinct", target))
            .unwrap() as usize
    }

    fn fully_qualified_table(relation_name: &str) -> (String, String) {
        let parts = relation_name
            .split('.')
            .map(|name| name.to_string())
            .collect::<Vec<String>>();

        let (schema_name, table_name) = match parts.len() {
            1 => (None, parts[0].clone()),
            2 => (Some(parts[0].clone()), parts[1].clone()),
            _ => error!(
                "Relation name \"{}\" is not parsable into schema name and table name",
                relation_name
            ),
        };

        match schema_name {
            None => {
                let table_count = Spi::get_one_with_args::<i64>("SELECT COUNT(*) FROM information_schema.tables WHERE table_name = $1 AND table_schema = 'public'", vec![
                    (PgBuiltInOids::TEXTOID.oid(), table_name.clone().into_datum())
                ]).unwrap();

                let error = format!("Relation \"{}\" could not be found in the public schema. Please specify the table schema, e.g. pgml.{}", table_name, table_name);

                match table_count {
                    0 => error!("{}", error),
                    1 => (String::from("public"), table_name),
                    _ => error!("{}", error),
                }
            }

            Some(schema_name) => {
                let exists = Spi::get_one_with_args::<i64>("SELECT COUNT(*) FROM information_schema.tables WHERE table_name = $1 AND table_schema = $2", vec![
                    (PgBuiltInOids::TEXTOID.oid(), table_name.clone().into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), schema_name.clone().into_datum()),
                ]).unwrap();

                if exists == 1 {
                    (schema_name, table_name)
                } else {
                    error!(
                        "Relation \"{}\".\"{}\" doesn't exist",
                        schema_name, table_name
                    );
                }
            }
        }
    }

    pub fn dataset(&mut self) -> Dataset {
        let numeric_encoded_dataset = self.numeric_encoded_dataset();
        let features = ndarray::ArrayView2::from_shape(
            (numeric_encoded_dataset.num_train_rows, numeric_encoded_dataset.num_features),
            &numeric_encoded_dataset.x_train,
        ).unwrap();
        let mut columns: Vec<usize> = Vec::with_capacity(numeric_encoded_dataset.num_features);
        self.columns.iter().for_each(|column| {
            if !column.label {
                for _ in 0..column.size {
                    columns.push(column.position); // TODO add array column indicators to combine stats
                }
            }
        });

        Zip::from(features.columns()) // TODO compute statistics for the label columns
            .and(&mut columns)
            .for_each(|data, position| {
                let mut sorted = data.to_vec();
                let column = &mut self.columns[*position - 1];
                sorted.sort_by(|a, b| a.total_cmp(&b));

                let mut statistics = &mut column.statistics;
                statistics.min = *sorted.first().unwrap();
                statistics.max = *sorted.last().unwrap();
                statistics.mean = sorted.iter().sum::<f32>() / sorted.len() as f32;
                statistics.median = sorted[sorted.len() / 2];
                statistics.variance = (sorted.iter().map(|i| {
                    let diff = statistics.mean - (*i);
                    diff * diff
                }).sum::<f32>() / sorted.len() as f32).sqrt();


                let mut i = 0;
                let histogram_boundaries = ndarray::Array::linspace(statistics.min, statistics.max, 21);
                let mut h = 0;
                let mut ventile_size = sorted.len() as f32 / 20.;
                let mut streak = 1;
                let mut max_streak = 0;
                let mut previous = f32::NAN;
                for value in &sorted {
                    if value.is_nan() {
                        statistics.missing += 1;
                    } else if *value == previous {
                        streak += 1;
                    } else {
                        statistics.distinct += 1;
                        if streak > max_streak {
                            statistics.mode = *value;
                            max_streak = streak;
                        }
                        streak = 1;
                        previous = *value;
                    }

                    // histogram
                    while *value >= histogram_boundaries[h] && h < statistics.histogram.len() {
                        h += 1;
                    }
                    statistics.histogram[h - 1] += 1;

                    // ventiles
                    let v = (i as f32 / ventile_size) as usize;
                    statistics.ventiles[v] = *value;

                    i += 1;
                }
            });

        // TODO add a column for Impute::missing, move to num_features()
        let total_width = self.columns.iter().map(|column| {
            if column.is_categorical() {
                match column.preprocessor.encode {
                    Encode::label => column.size,
                    _ => column.size * column.statistics.distinct,
                }
            } else {
                column.size
            }
        }).sum::<usize>();

        // TODO extract to reusable function for predictions
        let processed_features = vec![0_f32, total_width * numeric_encoded_dataset.num_train_rows];
        let mut c = 0;
        Zip::from(features.columns())
            .and(&mut columns)
            .for_each(|data, position| {
                let column = &mut self.columns[*position - 1];
                let mut statistics = &mut column.statistics;

                if column.is_categorical() && column.preprocessor.impute == Impute::mean {
                    error!("Cannot impute `{:?}` for categorical variable `{:?}`. Did you mean to impute `mode`?", column.preprocessor.impute, column.name);
                }
                info!("imputing {}: {:?}", column.name, column.preprocessor.impute);
                let offset = c * data.len();
                for (i, &d) in data.enumerate() {
                    let mut value = d;
                    if value.is_nan() {
                        match &column.preprocessor.impute {
                            Impute::mean => value = statistics.mean,
                            Impute::median => value = statistics.median,
                            Impute::mode => value = statistics.mode,
                            Impute::min => value = statistics.min,
                            Impute::max => value = statistics.max,
                            Impute::zero => value = 0.,
                            Impute::error => error!("{} missing values for {}", statistics.missing, column.name),
                        }
                    }

                    match &column.preprocessor.encode {
                        Encode::label => {},
                        Encode::one_hot => {
                            for i in 0..column.statistics.distinct {
                                // TODO don't ignore scaling
                                if v == i as f32 {
                                    processed_features[c + i] = 1
                                }
                            }
                        },
                        Encode::target => {},
                        Encode::ordinal(values) => {},
                    }
                    if column.is_categorical() {
                    } else {
                        match column.preprocessor.scale {
                            _ => {}
                        }
                    }

                    for i in 0..data.len() {
                        processed_features[offset + i] = data[i];
                    }
                    c += 1;
                }
            });

        info!(
            "Snapshot columns: {:?}",
            self.columns
        );
        info!(
            "Snapshot features: {:?}",
            numeric_encoded_dataset.x_train
        );
        numeric_encoded_dataset
    }

    // Encodes the raw training dataset
    // - replacing TEXT with label ids.
    pub fn numeric_encoded_dataset(&mut self) -> Dataset {
        let mut data = None;
        Spi::connect(|client| {
            let sql = format!(
                "SELECT {} FROM {} {}",
                self.columns
                    .iter()
                    .map(|c| c.quoted_name())
                    .collect::<Vec<String>>()
                    .join(", "),
                self.relation_name(),
                match self.materialized {
                    // If the snapshot is materialized, we already randomized it.
                    true => "",
                    false => {
                        if self.test_sampling == Sampling::random {
                            "ORDER BY random()"
                        } else {
                            ""
                        }
                    }
                },
            );

            // Postgres Arrays arrays are 1 indexed and so are SPI tuples...
            let result = client.select(&sql, None, None);
            let num_rows = result.len();

            let num_test_rows = if self.test_size > 1.0 {
                self.test_size as usize
            } else {
                (num_rows as f32 * self.test_size).round() as usize
            };

            let num_train_rows = num_rows - num_test_rows;
            if num_train_rows == 0 {
                error!(
                    "test_size = {} is too large. There are only {} samples.",
                    num_test_rows, num_rows
                );
            }

            let num_features = self.num_features();
            let num_labels = self.num_labels();

            let mut x_train: Vec<f32> = Vec::with_capacity(num_train_rows * num_features);
            let mut y_train: Vec<f32> = Vec::with_capacity(num_train_rows * num_labels);
            let mut x_test: Vec<f32> = Vec::with_capacity(num_test_rows * num_features);
            let mut y_test: Vec<f32> = Vec::with_capacity(num_test_rows * num_labels);

            // result: SpiTupleTable
            // row: SpiHeapTupleData
            // row[i]: SpiHeapTupleDataEntry
            result.enumerate().for_each(|(i, row)| {
                for column in &mut self.columns {
                    let vector = if column.label {
                        if i < num_train_rows {
                            &mut y_train
                        } else {
                            &mut y_test
                        }
                    } else if i < num_train_rows {
                        &mut x_train
                    } else {
                        &mut x_test
                    };
                    match column.pg_type.as_str() {
                        "bool" => {
                            vector.push(row[column.position].value::<bool>().unwrap() as u8 as f32)
                        }
                        "bool[]" => {
                            let vec = row[column.position].value::<Vec<bool>>().unwrap();
                            check_column_size(column, vec.len());
                            for j in vec {
                                vector.push(j as u8 as f32)
                            }
                        }
                        "bpchar" => {
                            vector.push(row[column.position].value::<i8>().unwrap() as f32)
                        }
                        "bpchar[]" => {
                            let vec = row[column.position].value::<Vec<i8>>().unwrap();
                            check_column_size(column, vec.len());

                            for j in vec {
                                vector.push(j as u8 as f32)
                            }
                        }
                        "int2" => vector.push(row[column.position].value::<i16>().unwrap() as f32),
                        "int2[]" => {
                            let vec = row[column.position].value::<Vec<i16>>().unwrap();
                            check_column_size(column, vec.len());

                            for j in vec {
                                vector.push(j as f32)
                            }
                        }
                        "int4" => vector.push(row[column.position].value::<i32>().unwrap() as f32),
                        "int4[]" => {
                            let vec = row[column.position].value::<Vec<i32>>().unwrap();
                            check_column_size(column, vec.len());

                            for j in vec {
                                vector.push(j as f32)
                            }
                        }
                        "int8" => vector.push(row[column.position].value::<i64>().unwrap() as f32),
                        "int8[]" => {
                            let vec = row[column.position].value::<Vec<i64>>().unwrap();
                            check_column_size(column, vec.len());

                            for j in vec {
                                vector.push(j as f32)
                            }
                        }
                        "float4" => {
                            vector.push(row[column.position].value::<f32>().unwrap() as f32)
                        }
                        "float4[]" => {
                            let vec = row[column.position].value::<Vec<f32>>().unwrap();
                            check_column_size(column, vec.len());

                            for j in vec {
                                vector.push(j as f32)
                            }
                        }
                        "float8" => {
                            vector.push(row[column.position].value::<f64>().unwrap() as f32)
                        }
                        "float8[]" => {
                            let vec = row[column.position].value::<Vec<f64>>().unwrap();
                            check_column_size(column, vec.len());

                            for j in vec {
                                vector.push(j as f32)
                            }
                        }
                        "text" | "varchar" => {
                            // label encode text on the fly for memory efficiency
                            let text = row[column.position].value::<String>().unwrap();
                            let value = (column.statistics.categories.len() + 1) as f32;
                            let category = column.statistics.categories.entry(text).or_insert(Category { value, members: 0 });
                            category.members += 1;
                            vector.push(category.value);
                        }
                        _ => error!("unhandled type: `{}` for `{}`", column.pg_type, column.name),
                    }
                }
            });

            let num_features = self.num_features();
            let num_labels = self.num_labels();

            let label = self.columns.iter().find(|c| c.name == self.y_column_name[0]).unwrap();
            let num_distinct_labels = label.statistics.categories.len();
            data = Some(Dataset {
                x_train,
                y_train,
                x_test,
                y_test,
                num_features,
                num_labels,
                num_rows,
                num_test_rows,
                num_train_rows,
                num_distinct_labels,
            });

            Ok(Some(()))
        });

        let data = data.unwrap();

        info!("{}", data);

        data
    }

    pub fn snapshot_name(&self) -> String {
        format!("\"pgml\".\"snapshot_{}\"", self.id)
    }

    pub fn relation_name(&self) -> String {
        match self.materialized {
            true => self.snapshot_name(),
            false => self.relation_name.clone(),
        }
    }
}

fn check_column_size(column: &mut Column, len: usize) {
    if column.size == 0 {
        column.size = len;
    } else if column.size != len {
        error!("Mismatched array length for feature `{}`. Expected: {} Received: {}", column.name, column.size, len);
    }
}
