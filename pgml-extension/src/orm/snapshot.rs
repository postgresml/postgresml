use std::cmp::Ordering;
use std::collections::btree_map::Entry;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;
use std::time::Instant;
use std::collections::HashMap;
use std::hash::BuildHasherDefault;

use hashers::fx_hash::FxHasher;
use indexmap::IndexMap;
use pgx::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ndarray_stats::QuantileExt;

use crate::orm::Dataset;
use crate::orm::Sampling;
use crate::orm::Status;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Ventile {
    tile: usize,
    min: f32,
    max: f32,
    members: usize,
}

impl PartialEq for Ventile {
    fn eq(&self, other: &Self) -> bool {
        self.tile == other.tile
    }
}

impl Eq for Ventile {}


#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct Category {
    pub(crate) id: usize,
    pub(crate) members: usize,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) enum Statistics {
    Continuous {
        nulls: usize,
        ventiles: Vec<Ventile>,
    },
    IntegerCategorical {
        nulls: usize,
        categories: Vec<usize>,
    },
    TextCategorical {
        nulls: usize,
        categories: HashMap<String, Category>,
    },
    Array {
        nulls: usize,
    }
}

#[derive(Debug, Eq, Serialize, Deserialize, Clone)]
pub(crate) struct Column {
    name: String,
    pg_type: String,
    nullable: bool,
    label: bool,
    position: usize,
    size: usize,
    pub(crate) statistics: Statistics,
}

impl Column {
    fn quoted_name(&self) -> String {
        format!(r#""{}""#, self.name)
    }

    fn stats_safe_name(&self) -> String {
        match self.pg_type.as_str() {
            "bool" => self.quoted_name() + "::INT4",
            "bool[]" => self.quoted_name() + "::INT4[]",
            _ => self.quoted_name(),
        }
    }
}

impl PartialEq<Self> for Column {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialOrd<Self> for Column {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Column {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position.cmp(&other.position)
    }
}

#[derive(Debug, Clone)]
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
    ) -> Snapshot {
        let mut snapshot: Option<Snapshot> = None;
        let status = Status::in_progress;

        // Validate table exists.
        let (schema_name, table_name) = Self::fully_qualified_table(relation_name);

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
                    pg_type = pg_type[1..].to_string() + "[]";
                    size = 0;
                }
                let nullable = row[3].value::<bool>().unwrap();
                let position = row[4].value::<i32>().unwrap() as usize;
                let label = y_column_name.contains(&name);
                let statistics = match pg_type.as_str() {
                    "float4" | "float8" => {
                        Statistics::Continuous {
                            nulls: 0,
                            ventiles: Vec::with_capacity(20),
                        }
                    },
                    "bool" | "bpchar" | "int2" | "int4" | "int8"  => {
                        Statistics::IntegerCategorical {
                            nulls: 0,
                            categories: Vec::new(),
                        }
                    },
                    "text" | "varchar" => {
                        Statistics::TextCategorical {
                            nulls: 0,
                            categories: HashMap::new(),
                        }
                    },
                    "bool[]" | "int2[]" | "int4[]" | "int8[]" | "float4[]" | "float8[]" => {
                        Statistics::Array {
                            nulls: 0,
                        }
                    }
                    _ => {
                        error!("unhandled type: `{}` for `{}`", pg_type, name);
                    }
                };
                columns.push(
                    Column {
                        name,
                        pg_type,
                        nullable,
                        label,
                        position,
                        size,
                        statistics
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
                    (PgBuiltInOids::BOOLOID.oid(), materialized.clone().into_datum()),
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
            s.analyze();
            snapshot = Some(s);
            Ok(Some(1))
        });

        snapshot.unwrap()
    }

    pub fn num_labels(&self) -> usize {
        self.y_column_name.len()
    }

    pub fn num_features(&self) -> usize {
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

    #[allow(clippy::format_push_string)]
    fn analyze(&mut self) {
        let now = Instant::now();
        let raw_dataset = self.dataset();

        let mut features = ndarray::ArrayView2::from_shape(
            (raw_dataset.num_train_rows, raw_dataset.num_features),
            &raw_dataset.x_train,
        )
            .unwrap();

        let mut c = 0;
        let mut a = 0;
        for data in features.columns() {
            let column = &mut self.columns[c];
            info!("c: {} a: {} column: {:?} data: {:?}", c, a, column, data);

            match &mut column.statistics {
                Statistics::Continuous { nulls, ventiles } => {
                    c += 1;
                    let data = data.to_vec().sort_by(|a,b| a.partial_cmp(b).unwrap());

                },
                Statistics::IntegerCategorical { nulls, ref mut categories } => {
                    c += 1;
                    let mut cs = vec![0; *data.max().unwrap() as usize + 1];
                    for i in data {
                        cs[*i as usize] += 1;
                    }
                    categories.extend(cs);
                },
                Statistics::TextCategorical {nulls, categories} => {
                    c += 1;
                },
                Statistics::Array { nulls } => {
                    if a < column.size {
                        a += 1;
                    } else {
                        c += 1;
                        a = 0;
                    }
                }
            }
        }

        // for column in &self.columns {
        //     match column.statistics {
        //         Statistics::Continuous(nulls, ventiles) => {}
        //         Statistics::IntegerCategorical(nulls, categories) => {},
        //         Statistics::TextCategorical(nulls, categories) => {},
        //         Statistics::Array(nulls, size) => {}
        //         "bpchar" | "bool" | "int2" | "int4" | "int8" => {
        //             // categoricals
        //         },
        //         "float4" | "float8" => {
        //             // fields.push(format!("{name}_min"));
        //             // fields.push(format!("{name}_max"));
        //             // fields.push(format!("{name}_mean"));
        //             // fields.push(format!("{name}_stddev"));
        //             // fields.push(format!("{name}_p25"));
        //             // fields.push(format!("{name}_p50"));
        //             // fields.push(format!("{name}_p75"));
        //             // fields.push(format!("{name}_count"));
        //             // fields.push(format!("{name}_distinct"));
        //             // fields.push(format!("{name}_nulls"));
        //         }
        //         "bool[]" | "int2[]" | "int4[]" | "int8[]" | "float4[]" | "float8[]" => {
        //             // fields.push(format!("{name}_dims"));
        //             // fields.push(format!("{name}_cardinality"));
        //             // fields.push(format!("{name}_min"));
        //             // fields.push(format!("{name}_max"));
        //             // fields.push(format!("{name}_nulls"));
        //         },
        //         "text" | "varchar" => {
        //             // fields.push(format!("{name}_nulls"));
        //         }
        //         _ => error!("unhandled type: `{}` for `{}`", column.pg_type, column.name)
        //     }
        // }

        Spi::connect(|client| {
            client.select("UPDATE pgml.snapshots SET status = $1::pgml.status, columns = $2 WHERE id = $3", Some(1), Some(vec![
                (PgBuiltInOids::TEXTOID.oid(), Status::successful.to_string().into_datum()),
                (PgBuiltInOids::JSONBOID.oid(), JsonB(json!(self.columns)).into_datum()),
                (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
            ]));

            Ok(Some(1))
        });
    }

    pub fn dataset(&mut self) -> Dataset {
        self.raw_dataset()
    }

    pub fn raw_dataset(&mut self) -> Dataset {
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
                            if column.size == 0 {
                                column.size = vec.len();
                            } else if column.size != vec.len() {
                                error!("Mismatched array length for feature `{}`. Expected: {} Received: {}", column.name, column.size, vec.len());
                            }

                            for j in vec {
                                vector.push(j as u8 as f32)
                            }
                        }
                        "bpchar" => {
                            vector.push(row[column.position].value::<i8>().unwrap() as f32)
                        }
                        "bpchar[]" => {
                            let vec = row[column.position].value::<Vec<i8>>().unwrap();
                            if column.size == 0 {
                                column.size = vec.len();
                            } else if column.size != vec.len() {
                                error!("Mismatched array length for feature `{}`. Expected: {} Received: {}", column.name, column.size, vec.len());
                            }

                            for j in vec {
                                vector.push(j as u8 as f32)
                            }
                        }
                        "int2" => vector.push(row[column.position].value::<i16>().unwrap() as f32),
                        "int2[]" => {
                            let vec = row[column.position].value::<Vec<i16>>().unwrap();
                            if column.size == 0 {
                                column.size = vec.len();
                            } else if column.size != vec.len() {
                                error!("Mismatched array length for feature `{}`. Expected: {} Received: {}", column.name, column.size, vec.len());
                            }

                            for j in vec {
                                vector.push(j as f32)
                            }
                        }
                        "int4" => vector.push(row[column.position].value::<i32>().unwrap() as f32),
                        "int4[]" => {
                            let vec = row[column.position].value::<Vec<i32>>().unwrap();
                            if column.size == 0 {
                                column.size = vec.len();
                            } else if column.size != vec.len() {
                                error!("Mismatched array length for feature `{}`. Expected: {} Received: {}", column.name, column.size, vec.len());
                            }

                            for j in vec {
                                vector.push(j as f32)
                            }
                        }
                        "int8" => vector.push(row[column.position].value::<i64>().unwrap() as f32),
                        "int8[]" => {
                            let vec = row[column.position].value::<Vec<i64>>().unwrap();
                            if column.size == 0 {
                                column.size = vec.len();
                            } else if column.size != vec.len() {
                                error!("Mismatched array length for feature `{}`. Expected: {} Received: {}", column.name, column.size, vec.len());
                            }

                            for j in vec {
                                vector.push(j as f32)
                            }
                        }
                        "float4" => {
                            vector.push(row[column.position].value::<f32>().unwrap() as f32)
                        }
                        "float4[]" => {
                            let vec = row[column.position].value::<Vec<f32>>().unwrap();
                            if column.size == 0 {
                                column.size = vec.len();
                            } else if column.size != vec.len() {
                                error!("Mismatched array length for feature `{}`. Expected: {} Received: {}", column.name, column.size, vec.len());
                            }

                            for j in vec {
                                vector.push(j as f32)
                            }
                        }
                        "float8" => {
                            vector.push(row[column.position].value::<f64>().unwrap() as f32)
                        }
                        "float8[]" => {
                            let vec = row[column.position].value::<Vec<f64>>().unwrap();
                            if column.size == 0 {
                                column.size = vec.len();
                            } else if column.size != vec.len() {
                                error!("Mismatched array length for feature `{}`. Expected: {} Received: {}", column.name, column.size, vec.len());
                            }

                            for j in vec {
                                vector.push(j as f32)
                            }
                        }
                        "text" | "varchar" => {
                            // we handle text categorical encoding on the fly for memory efficiency
                            let text = row[column.position].value::<String>().unwrap();
                            let id = match column.statistics {
                                Statistics::TextCategorical { nulls, ref mut categories } => {
                                    let id = categories.len() + 1;
                                    let values = categories.entry(text).or_insert(Category {id: id, members: 0 });
                                    values.members += 1;
                                    values.id
                                }
                                _ => error!("non text categorical stats for text column")
                            };
                            vector.push(id as f32);
                        }
                        _ => error!("unhandled type: `{}` for `{}`", column.pg_type, column.name),
                    }
                }
            });

            info!(
                "Snapshot columns: {:?}",
                self.columns
            );
            info!(
                "Snapshot features: {:?}",
                x_train
            );

            let num_features = self.num_features();
            let num_labels = self.num_labels();

            let label = self.columns.iter().find(|c| c.name == self.y_column_name[0]).unwrap();
            let num_distinct_labels = match &label.statistics {
                Statistics::Continuous { nulls, ventiles } => 0,
                Statistics::IntegerCategorical { nulls, categories } => categories.len(),
                Statistics::TextCategorical {nulls, categories} => categories.len(),
                Statistics::Array { nulls } => 0,
            };

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
