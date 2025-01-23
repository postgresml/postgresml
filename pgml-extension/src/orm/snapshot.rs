use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

use indexmap::IndexMap;
use ndarray::Zip;
use pgrx::{datum::*, *};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::orm::Sampling;
use crate::orm::Status;
use crate::orm::{ConversationDataset, Dataset, TextClassificationDataset, TextPairClassificationDataset};

// Categories use a designated string to represent NULL categorical values,
// rather than Option<String> = None, because the JSONB serialization schema
// only supports String keys in JSON objects, unlike a Rust HashMap.
pub(crate) const NULL_CATEGORY_KEY: &str = "__NULL__";

// A category maintains the encoded value for a key, as well as counting the number
// of members in the training set for statistical purposes, e.g. target_mean
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct Category {
    pub(crate) value: f32,
    pub(crate) members: usize,
}

// Statistics are computed for every column over the training data when the
// data is read.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct Statistics {
    min: f32,
    max: f32,
    max_abs: f32,
    mean: f32,
    median: f32,
    mode: f32,
    variance: f32,
    std_dev: f32,
    missing: usize,
    distinct: usize,
    histogram: Vec<usize>,
    ventiles: Vec<f32>,
    pub categories: Option<HashMap<String, Category>>,
}

impl Default for Statistics {
    fn default() -> Self {
        Statistics {
            min: f32::NAN,
            max: f32::NAN,
            max_abs: f32::NAN,
            mean: f32::NAN,
            median: f32::NAN,
            mode: f32::NAN,
            variance: f32::NAN,
            std_dev: f32::NAN,
            missing: 0,
            distinct: 0,
            histogram: vec![0; 20],
            ventiles: vec![f32::NAN; 19],
            categories: None,
        }
    }
}

// How to encode categorical values
// TODO add limit and min_frequency params to all
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
pub(crate) enum Encode {
    // For use with algorithms that directly support the data type
    #[default]
    native,
    // Encode each category as the mean of the target
    target,
    // Encode each category as one boolean column per category
    one_hot,
    // Encode each category as ascending integer values
    ordinal(Vec<String>),
}

// How to replace missing values
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
pub(crate) enum Impute {
    #[default]
    // Raises an error at runtime
    error,
    mean,
    median,
    mode,
    min,
    max,
    // Replaces with 0
    zero,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
pub(crate) enum Scale {
    #[default]
    preserve,
    standard,
    min_max,
    max_abs,
    robust,
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
pub struct Column {
    pub(crate) name: String,
    pub(crate) pg_type: String,
    pub(crate) nullable: bool,
    pub(crate) label: bool,
    pub(crate) position: usize,
    pub(crate) size: usize,
    pub(crate) array: bool,
    pub(crate) preprocessor: Preprocessor,
    pub(crate) statistics: Statistics,
}

impl Column {
    fn categorical_type(pg_type: &str) -> bool {
        Column::nominal_type(pg_type) || Column::ordinal_type(pg_type)
    }

    fn ordinal_type(_pg_type: &str) -> bool {
        false
    }

    fn nominal_type(pg_type: &str) -> bool {
        matches!(
            pg_type,
            "bpchar" | "text" | "varchar" | "bpchar[]" | "text[]" | "varchar[]"
        )
    }

    pub(crate) fn quoted_name(&self) -> String {
        format!(r#""{}""#, self.name)
    }

    #[inline]
    pub(crate) fn get_category_value(&self, key: &str) -> f32 {
        match self.statistics.categories.as_ref().unwrap().get(key) {
            Some(category) => category.value,
            None => f32::NAN,
        }
    }

    #[inline]
    pub(crate) fn scale(&self, value: f32) -> f32 {
        match self.preprocessor.scale {
            Scale::standard => (value - self.statistics.mean) / self.statistics.std_dev,
            Scale::min_max => (value - self.statistics.min) / (self.statistics.max - self.statistics.min),
            Scale::max_abs => value / self.statistics.max_abs,
            Scale::robust => {
                (value - self.statistics.median) / (self.statistics.ventiles[15] - self.statistics.ventiles[5])
            }
            Scale::preserve => value,
        }
    }

    #[inline]
    pub(crate) fn impute(&self, value: f32) -> f32 {
        if value.is_nan() {
            match &self.preprocessor.impute {
                Impute::mean => self.statistics.mean,
                Impute::median => self.statistics.median,
                Impute::mode => self.statistics.mode,
                Impute::min => self.statistics.min,
                Impute::max => self.statistics.max,
                Impute::zero => 0.,
                Impute::error => error!("{} missing values for {}. You may provide a preprocessor to impute a value. e.g:\n\n pgml.train(preprocessor => '{{{:?}: {{\"impute\": \"mean\"}}}}'", self.statistics.missing, self.name, self.name),
            }
        } else {
            value
        }
    }

    pub(crate) fn encoded_width(&self) -> usize {
        match self.preprocessor.encode {
            Encode::one_hot => self.statistics.categories.as_ref().unwrap().len() - 1,
            _ => 1,
        }
    }

    pub(crate) fn array_width(&self) -> usize {
        self.size
    }

    pub(crate) fn preprocess(
        &self,
        data: &ndarray::ArrayView<f32, ndarray::Ix1>,
        processed_data: &mut [f32],
        features_width: usize,
        position: usize,
    ) {
        for (row, &d) in data.iter().enumerate() {
            let value = self.impute(d);
            match &self.preprocessor.encode {
                Encode::one_hot => {
                    for i in 0..self.statistics.categories.as_ref().unwrap().len() - 1 {
                        let one_hot = if i == value as usize { 1. } else { 0. } as f32;
                        processed_data[row * features_width + position + i] = one_hot;
                    }
                }
                _ => processed_data[row * features_width + position] = self.scale(value),
            };
        }
    }

    fn analyze(
        &mut self,
        array: &ndarray::ArrayView<f32, ndarray::Ix1>,
        target: &ndarray::ArrayView<f32, ndarray::Ix1>,
    ) {
        // target encode if necessary before analyzing
        if self.preprocessor.encode == Encode::target {
            let categories = self.statistics.categories.as_mut().unwrap();
            let mut sums = vec![0_f32; categories.len() + 1];
            let mut total = 0.;
            Zip::from(array).and(target).for_each(|&value, &target| {
                total += target;
                sums[value as usize] += target;
            });
            let avg_target = total / categories.len() as f32;
            for category in categories.values_mut() {
                if category.members > 0 {
                    let sum = sums[category.value as usize];
                    category.value = sum / category.members as f32;
                } else {
                    // use avg target for categories w/ no members, e.g. __NULL__ category in a complete dataset
                    category.value = avg_target;
                }
            }
        }

        // Data is filtered for NaN because it is not well-defined statistically, and they are counted as separate stat
        let mut data = array
            .iter()
            .filter_map(|n| if n.is_nan() { None } else { Some(*n) })
            .collect::<Vec<f32>>();
        data.sort_by(|a, b| a.total_cmp(b));

        // FixMe: Arrays are analyzed many times, clobbering/appending to the same stats, columns are also re-analyzed in memory during tests, which can cause unnexpected failures
        let statistics = &mut self.statistics;
        statistics.min = *data.first().unwrap();
        statistics.max = *data.last().unwrap();
        statistics.max_abs = if statistics.min.abs() > statistics.max.abs() {
            statistics.min.abs()
        } else {
            statistics.max.abs()
        };
        statistics.mean = data.iter().sum::<f32>() / data.len() as f32;
        statistics.median = data[data.len() / 2];
        statistics.missing = array.len() - data.len();
        if self.label && statistics.missing > 0 {
            error!("The training data labels in \"{}\" contain {} NULL values. Consider filtering these values from the training data by creating a VIEW that includes a SQL filter like `WHERE {} IS NOT NULL`.", self.name, statistics.missing, self.name);
        }
        statistics.variance = data
            .iter()
            .map(|i| {
                let diff = statistics.mean - (*i);
                diff * diff
            })
            .sum::<f32>()
            / data.len() as f32;
        statistics.std_dev = statistics.variance.sqrt();
        let histogram_boundaries = ndarray::Array::linspace(statistics.min, statistics.max, 21);
        let mut h = 0;
        let ventile_size = data.len() as f32 / 20.;
        let mut streak = 1;
        let mut max_streak = 0;
        let mut previous = f32::NAN;

        let mut modes = Vec::new();
        statistics.distinct = 0; // necessary reset before array columns clobber
        for (i, &value) in data.iter().enumerate() {
            // mode candidates form streaks
            if value == previous {
                streak += 1;
            } else if !previous.is_nan() {
                match streak.cmp(&max_streak) {
                    Ordering::Greater => {
                        modes = vec![previous];
                        max_streak = streak;
                    }
                    Ordering::Equal => modes.push(previous),
                    Ordering::Less => {}
                }
                streak = 1;
                statistics.distinct += 1;
            }
            previous = value;

            // histogram
            while value >= histogram_boundaries[h] && h < statistics.histogram.len() {
                h += 1;
            }
            statistics.histogram[h - 1] += 1;

            // ventiles
            let v = (i as f32 / ventile_size) as usize;
            if v < 19 {
                statistics.ventiles[v] = value;
            }
        }
        // Pick the mode in the middle of all the candidates with the longest streaks
        if !previous.is_nan() {
            statistics.distinct += 1;
            if streak > max_streak {
                statistics.mode = previous;
            } else {
                statistics.mode = modes[modes.len() / 2];
            }
        }

        // Fill missing ventiles with the preceding value, when there are fewer than 20 points
        for i in 1..statistics.ventiles.len() {
            if statistics.ventiles[i].is_nan() {
                statistics.ventiles[i] = statistics.ventiles[i - 1];
            }
        }

        info!("Column {:?}: {:?}", self.name, statistics);
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

// Array and one hot encoded columns take up multiple positions in a feature row
#[derive(Debug, Clone)]
pub struct ColumnRowPosition {
    pub(crate) column_position: usize,
    pub(crate) row_position: usize,
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
    pub(crate) feature_positions: Vec<ColumnRowPosition>,
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
                .unwrap()
                .first();
            if !result.is_empty() {
                let jsonb: JsonB = result.get(7).unwrap().unwrap();
                let columns: Vec<Column> =
                    serde_json::from_value(jsonb.0).expect("invalid json description of columns");
                // let jsonb: JsonB = result.get(8).unwrap();
                // let analysis: Option<IndexMap<String, f32>> = Some(serde_json::from_value(jsonb.0).unwrap());
                let mut s = Snapshot {
                    id: result.get(1).unwrap().unwrap(),
                    relation_name: result.get(2).unwrap().unwrap(),
                    y_column_name: result.get(3).unwrap().unwrap_or_default(),
                    test_size: result.get(4).unwrap().unwrap(),
                    test_sampling: Sampling::from_str(result.get(5).unwrap().unwrap()).unwrap(),
                    status: Status::from_str(result.get(6).unwrap().unwrap()).unwrap(),
                    columns,
                    analysis: None,
                    created_at: result.get(9).unwrap().unwrap(),
                    updated_at: result.get(10).unwrap().unwrap(),
                    materialized: result.get(11).unwrap().unwrap(),
                    feature_positions: Vec::new(),
                };
                s.feature_positions = s.feature_positions();
                snapshot = Some(s)
            }
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
                    Some(vec![(PgBuiltInOids::INT8OID.oid(), project_id.into_datum())]),
                )
                .unwrap()
                .first();
            if !result.is_empty() {
                let jsonb: JsonB = result.get(7).unwrap().unwrap();
                let columns: Vec<Column> = serde_json::from_value(jsonb.0).unwrap();
                let jsonb: JsonB = result.get(8).unwrap().unwrap();
                let analysis: Option<IndexMap<String, f32>> = Some(serde_json::from_value(jsonb.0).unwrap());

                let mut s = Snapshot {
                    id: result.get(1).unwrap().unwrap(),
                    relation_name: result.get(2).unwrap().unwrap(),
                    y_column_name: result.get(3).unwrap().unwrap_or_default(),
                    test_size: result.get(4).unwrap().unwrap(),
                    test_sampling: Sampling::from_str(result.get(5).unwrap().unwrap()).unwrap(),
                    status: Status::from_str(result.get(6).unwrap().unwrap()).unwrap(),
                    columns,
                    analysis,
                    created_at: result.get(9).unwrap().unwrap(),
                    updated_at: result.get(10).unwrap().unwrap(),
                    materialized: result.get(11).unwrap().unwrap(),
                    feature_positions: Vec::new(),
                };
                s.feature_positions = s.feature_positions();
                snapshot = Some(s)
            }
        });
        snapshot
    }

    pub fn create(
        relation_name: &str,
        y_column_name: Option<Vec<String>>,
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

        let mut position = 0; // Postgres column positions are not updated when other columns are dropped, but we expect consecutive positions when we read the table.
        Spi::connect(|mut client| {
            let mut columns: Vec<Column> = Vec::new();
            client.select("SELECT column_name::TEXT, udt_name::TEXT, is_nullable::BOOLEAN FROM information_schema.columns WHERE table_schema = $1 AND table_name = $2 ORDER BY ordinal_position ASC",
                None,
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), schema_name.into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), table_name.into_datum()),
                ]))
            .unwrap()
            .for_each(|row| {
                let name = row[1].value::<String>().unwrap().unwrap();
                let mut pg_type = row[2].value::<String>().unwrap().unwrap();
                let mut size = 1;
                let mut array = false;
                if pg_type.starts_with('_') {
                    size = 0;
                    array = true;
                    pg_type = pg_type[1..].to_string() + "[]";
                }
                let nullable = row[3].value::<bool>().unwrap().unwrap();
                position += 1;
                let label = match y_column_name {
                    Some(ref y_column_name) => y_column_name.contains(&name),
                    None => false,
                };
                let mut statistics = Statistics::default();
                let preprocessor = match preprocessors.get(&name) {
                    Some(preprocessor) => {
                        let preprocessor = preprocessor.clone();
                        if Column::categorical_type(&pg_type) {
                            if preprocessor.impute == Impute::mean && preprocessor.encode != Encode::target {
                                error!("Error initializing preprocessor for column: {:?}.\n\n  You can not specify {{\"impute: mean\"}} for a categorical variable unless it is also encoded using `target_mean`, because there is no \"average\" category. `{{\"impute: mode\"}}` is valid alternative, since there is a most common category. Another option would be to encode using target_mean, and then the target mean will be imputed for missing categoricals.", name);
                            }
                        } else if preprocessor.encode != Encode::native {
                            error!("Error initializing preprocessor for column: {:?}.\n\n  It does not make sense to {{\"encode: {:?}}} a continuous variable. Please use the default `native`.", name, preprocessor.scale);
                        }
                        preprocessor
                    },
                    None => Preprocessor::default(),
                };

                if Column::categorical_type(&pg_type) || preprocessor.encode != Encode::native {
                    let mut categories = HashMap::new();
                    categories.insert(
                        NULL_CATEGORY_KEY.to_string(),
                        Category {
                            value: 0.,
                            members: 0,
                        }
                    );
                    statistics.categories = Some(categories);
                }

                columns.push(
                    Column {
                        name,
                        pg_type,
                        nullable,
                        label,
                        position,
                        size,
                        array,
                        statistics,
                        preprocessor,
                    }
                );
            });

            if y_column_name.is_some() {
                for column in y_column_name.as_ref().unwrap() {
                    if !columns.iter().any(|c| c.label && &c.name == column) {
                        error!(
                            "Column `{}` not found. Did you pass the correct `y_column_name`?",
                            column
                        )
                    }
                }
            }

            let result = client.update("INSERT INTO pgml.snapshots (relation_name, y_column_name, test_size, test_sampling, status, columns, materialized) VALUES ($1, $2, $3, $4::pgml.sampling, $5::pgml.status, $6, $7) RETURNING id, relation_name, y_column_name, test_size, test_sampling::TEXT, status::TEXT, columns, analysis, created_at, updated_at;",
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
            ).unwrap().first();

            let s = Snapshot {
                id: result.get(1).unwrap().unwrap(),
                relation_name: result.get(2).unwrap().unwrap(),
                y_column_name: result.get(3).unwrap().unwrap_or_default(),
                test_size: result.get(4).unwrap().unwrap(),
                test_sampling: Sampling::from_str(result.get(5).unwrap().unwrap()).unwrap(),
                status,         // 6
                columns,        // 7
                analysis: None, // 8
                created_at: result.get(9).unwrap().unwrap(),
                updated_at: result.get(10).unwrap().unwrap(),
                materialized,
                feature_positions: Vec::new(),
            };

            if materialized {
                let sampled_query = s.test_sampling.get_sql(&s.relation_name, s.columns.clone());
                let sql = format!(r#"CREATE TABLE "pgml"."snapshot_{}" AS {}"#, s.id, sampled_query);
                client.update(&sql, None, None).unwrap();
            }
            snapshot = Some(s);
        });

        snapshot.unwrap()
    }

    pub(crate) fn labels(&self) -> impl Iterator<Item = &Column> {
        self.columns.iter().filter(|c| c.label)
    }

    pub(crate) fn label_positions(&self) -> Vec<ColumnRowPosition> {
        let mut label_positions = Vec::with_capacity(self.num_labels());
        let mut row_position = 0;
        for column in self.labels() {
            for _ in 0..column.size {
                label_positions.push(ColumnRowPosition {
                    column_position: column.position,
                    row_position,
                });
                row_position += column.encoded_width();
            }
        }
        label_positions
    }

    pub(crate) fn features(&self) -> impl Iterator<Item = &Column> {
        self.columns.iter().filter(|c| !c.label)
    }

    pub(crate) fn feature_positions(&self) -> Vec<ColumnRowPosition> {
        let mut feature_positions = Vec::with_capacity(self.num_features());
        let mut row_position = 0;
        for column in self.features() {
            for _ in 0..column.size {
                feature_positions.push(ColumnRowPosition {
                    column_position: column.position,
                    row_position,
                });
                row_position += column.encoded_width();
            }
        }
        feature_positions
    }

    pub(crate) fn num_labels(&self) -> usize {
        self.labels().map(|f| f.size).sum::<usize>()
    }

    pub(crate) fn first_label(&self) -> &Column {
        self.labels().find(|l| l.name == self.y_column_name[0]).unwrap()
    }

    pub(crate) fn num_classes(&self) -> usize {
        match &self.y_column_name.len() {
            0 => 0,
            _ => match &self.first_label().statistics.categories {
                Some(categories) => categories.len(),
                None => self.first_label().statistics.distinct,
            },
        }
    }

    pub(crate) fn num_features(&self) -> usize {
        self.features().map(|c| c.size).sum::<usize>()
    }

    pub(crate) fn features_width(&self) -> usize {
        self.features()
            .map(|f| f.array_width() * f.encoded_width())
            .sum::<usize>()
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
                let table_count = Spi::get_one_with_args::<i64>(
                    "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = $1 AND table_schema = 'public'",
                    vec![(PgBuiltInOids::TEXTOID.oid(), table_name.clone().into_datum())],
                )
                .unwrap()
                .unwrap();

                let error = format!("Relation \"{}\" could not be found in the public schema. Please specify the table schema, e.g. pgml.{}", table_name, table_name);

                match table_count {
                    0 => error!("{}", error),
                    1 => (String::from("public"), table_name),
                    _ => error!("{}", error),
                }
            }

            Some(schema_name) => {
                let exists = Spi::get_one_with_args::<i64>(
                    "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = $1 AND table_schema = $2",
                    vec![
                        (PgBuiltInOids::TEXTOID.oid(), table_name.clone().into_datum()),
                        (PgBuiltInOids::TEXTOID.oid(), schema_name.clone().into_datum()),
                    ],
                )
                .unwrap();

                if exists == Some(1) {
                    (schema_name, table_name)
                } else {
                    error!("Relation \"{}\".\"{}\" doesn't exist", schema_name, table_name);
                }
            }
        }
    }

    fn select_sql(&self) -> String {
        match self.materialized {
            true => {
                format!(
                    "SELECT {} FROM {}",
                    self.columns
                        .iter()
                        .map(|c| c.quoted_name())
                        .collect::<Vec<String>>()
                        .join(", "),
                    self.relation_name_quoted()
                )
            }
            false => self
                .test_sampling
                .get_sql(&self.relation_name_quoted(), self.columns.clone()),
        }
    }

    fn train_test_split(&self, num_rows: usize) -> (usize, usize) {
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

        (num_train_rows, num_test_rows)
    }

    pub fn text_classification_dataset(&mut self, dataset_args: default!(JsonB, "'{}'")) -> TextClassificationDataset {
        let mut data = None;

        Spi::connect(|client| {
            let result = client.select(&self.select_sql(), None, None).unwrap();
            let num_rows = result.len();
            let (num_train_rows, num_test_rows) = self.train_test_split(num_rows);
            let num_features = self.num_features();
            let num_labels = self.num_labels();

            let mut text_train: Vec<String> = Vec::with_capacity(num_train_rows);
            let mut class_train: Vec<String> = Vec::with_capacity(num_train_rows);
            let mut text_test: Vec<String> = Vec::with_capacity(num_test_rows);
            let mut class_test: Vec<String> = Vec::with_capacity(num_test_rows);

            let class_column_value = dataset_args
                .0
                .get("class_column")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "class".to_string());

            let text_column_value = dataset_args
                .0
                .get("text_column")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "text".to_string());

            result.enumerate().for_each(|(i, row)| {
                for column in &mut self.columns {
                    let vector = if column.name == text_column_value {
                        if i < num_train_rows {
                            &mut text_train
                        } else {
                            &mut text_test
                        }
                    } else if column.name == class_column_value {
                        if i < num_train_rows {
                            &mut class_train
                        } else {
                            &mut class_test
                        }
                    } else {
                        continue;
                    };

                    match column.pg_type.as_str() {
                        "bpchar" | "text" | "varchar" => match row[column.position].value::<String>().unwrap() {
                            Some(text) => vector.push(text),
                            None => error!("NULL training text is not handled"),
                        },
                        _ => error!("only text type columns are supported"),
                    }
                }
            });
            let num_distinct_labels = class_train.iter().cloned().collect::<HashSet<_>>().len();
            data = Some(TextClassificationDataset {
                text_train,
                class_train,
                text_test,
                class_test,
                num_features,
                num_labels,
                num_rows,
                num_test_rows,
                num_train_rows,
                // TODO rename and audit this
                num_distinct_labels,
            });

            Ok::<std::option::Option<()>, i64>(Some(())) // this return type is nonsense
        })
        .unwrap();

        let data = data.unwrap();

        info!("{}", data);

        data
    }

    pub fn text_pair_classification_dataset(
        &mut self,
        dataset_args: default!(JsonB, "'{}'"),
    ) -> TextPairClassificationDataset {
        let mut data = None;

        Spi::connect(|client| {
            let result = client.select(&self.select_sql(), None, None).unwrap();
            let num_rows = result.len();
            let (num_train_rows, num_test_rows) = self.train_test_split(num_rows);
            let num_features = 2;
            let num_labels = self.num_labels();

            let mut text1_train: Vec<String> = Vec::with_capacity(num_train_rows);
            let mut text2_train: Vec<String> = Vec::with_capacity(num_train_rows);
            let mut class_train: Vec<String> = Vec::with_capacity(num_train_rows);
            let mut text1_test: Vec<String> = Vec::with_capacity(num_test_rows);
            let mut text2_test: Vec<String> = Vec::with_capacity(num_test_rows);
            let mut class_test: Vec<String> = Vec::with_capacity(num_test_rows);

            let text1_column_value = dataset_args
                .0
                .get("text1_column")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "text1".to_string());

            let text2_column_value = dataset_args
                .0
                .get("text2_column")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "text2".to_string());

            let class_column_value = dataset_args
                .0
                .get("class_column")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "class".to_string());

            result.enumerate().for_each(|(i, row)| {
                for column in &mut self.columns {
                    let vector = if column.name == text1_column_value {
                        if i < num_train_rows {
                            &mut text1_train
                        } else {
                            &mut text1_test
                        }
                    } else if column.name == text2_column_value {
                        if i < num_train_rows {
                            &mut text2_train
                        } else {
                            &mut text2_test
                        }
                    } else if column.name == class_column_value {
                        if i < num_train_rows {
                            &mut class_train
                        } else {
                            &mut class_test
                        }
                    } else {
                        continue;
                    };

                    match column.pg_type.as_str() {
                        "bpchar" | "text" | "varchar" => match row[column.position].value::<String>().unwrap() {
                            Some(text) => vector.push(text),
                            None => error!("NULL training text is not handled"),
                        },
                        _ => error!("only text type columns are supported"),
                    }
                }
            });

            let num_distinct_labels = class_train.iter().cloned().collect::<HashSet<_>>().len();
            data = Some(TextPairClassificationDataset {
                text1_train,
                text2_train,
                class_train,
                text1_test,
                text2_test,
                class_test,
                num_features,
                num_labels,
                num_rows,
                num_test_rows,
                num_train_rows,
                // TODO rename and audit this
                num_distinct_labels,
            });

            Ok::<std::option::Option<()>, i64>(Some(())) // this return type is nonsense
        })
        .unwrap();

        let data = data.unwrap();

        info!("{}", data);

        data
    }

    pub fn conversation_dataset(&mut self, dataset_args: default!(JsonB, "'{}'")) -> ConversationDataset {
        let mut data = None;

        Spi::connect(|client| {
            let result = client.select(&self.select_sql(), None, None).unwrap();
            let num_rows = result.len();
            let (num_train_rows, num_test_rows) = self.train_test_split(num_rows);
            let num_features = 2;

            let mut system_train: Vec<String> = Vec::with_capacity(num_train_rows);
            let mut user_train: Vec<String> = Vec::with_capacity(num_train_rows);
            let mut assistant_train: Vec<String> = Vec::with_capacity(num_train_rows);
            let mut system_test: Vec<String> = Vec::with_capacity(num_test_rows);
            let mut user_test: Vec<String> = Vec::with_capacity(num_test_rows);
            let mut assistant_test: Vec<String> = Vec::with_capacity(num_test_rows);

            let system_column_value = dataset_args
                .0
                .get("system_column")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "system".to_string());

            let user_column_value = dataset_args
                .0
                .get("user_column")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "user".to_string());

            let assistant_column_value = dataset_args
                .0
                .get("assistant_column")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "assistant".to_string());

            result.enumerate().for_each(|(i, row)| {
                for column in &mut self.columns {
                    let vector = if column.name == system_column_value {
                        if i < num_train_rows {
                            &mut system_train
                        } else {
                            &mut system_test
                        }
                    } else if column.name == user_column_value {
                        if i < num_train_rows {
                            &mut user_train
                        } else {
                            &mut user_test
                        }
                    } else if column.name == assistant_column_value {
                        if i < num_train_rows {
                            &mut assistant_train
                        } else {
                            &mut assistant_test
                        }
                    } else {
                        continue;
                    };

                    match column.pg_type.as_str() {
                        "bpchar" | "text" | "varchar" => match row[column.position].value::<String>().unwrap() {
                            Some(text) => vector.push(text),
                            None => error!("NULL training text is not handled"),
                        },
                        _ => error!("only text type columns are supported"),
                    }
                }
            });

            data = Some(ConversationDataset {
                system_train,
                user_train,
                assistant_train,
                system_test,
                user_test,
                assistant_test,
                num_features,
                num_rows,
                num_test_rows,
                num_train_rows,
            });

            Ok::<std::option::Option<()>, i64>(Some(())) // this return type is nonsense
        })
        .unwrap();

        let data = data.unwrap();

        info!("{}", data);

        data
    }

    pub fn tabular_dataset(&mut self) -> Dataset {
        let numeric_encoded_dataset = self.numeric_encoded_dataset();

        let label_data = ndarray::ArrayView2::from_shape(
            (
                numeric_encoded_dataset.num_train_rows,
                numeric_encoded_dataset.num_labels,
            ),
            &numeric_encoded_dataset.y_train,
        )
        .unwrap();

        let feature_data = ndarray::ArrayView2::from_shape(
            (
                numeric_encoded_dataset.num_train_rows,
                numeric_encoded_dataset.num_features,
            ),
            &numeric_encoded_dataset.x_train,
        )
        .unwrap();

        // We only analyze supervised training sets that have labels for now.
        if numeric_encoded_dataset.num_labels > 0 {
            // We only analyze features against the first label in joint regression.
            let target_data = label_data.columns().into_iter().next().unwrap();

            // Analyze labels
            Zip::from(label_data.columns())
                .and(&self.label_positions())
                .for_each(|data, position| {
                    let column = &mut self.columns[position.column_position - 1]; // lookup the mutable one
                    column.analyze(&data, &target_data);
                });

            // Analyze features
            Zip::from(feature_data.columns())
                .and(&self.feature_positions())
                .for_each(|data, position| {
                    let column = &mut self.columns[position.column_position - 1]; // lookup the mutable one
                    column.analyze(&data, &target_data);
                });
        } else {
            // Analyze features for unsupervised learning
            Zip::from(feature_data.columns())
                .and(&self.feature_positions())
                .for_each(|data, position| {
                    let column = &mut self.columns[position.column_position - 1]; // lookup the mutable one
                    column.analyze(&data, &data);
                });
        }

        let mut analysis = IndexMap::new();
        analysis.insert("samples".to_string(), numeric_encoded_dataset.num_rows as f32);
        self.analysis = Some(analysis);

        // Record the analysis
        Spi::run_with_args(
            "UPDATE pgml.snapshots SET analysis = $1, columns = $2 WHERE id = $3",
            Some(vec![
                (PgBuiltInOids::JSONBOID.oid(), JsonB(json!(self.analysis)).into_datum()),
                (PgBuiltInOids::JSONBOID.oid(), JsonB(json!(self.columns)).into_datum()),
                (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
            ]),
        )
        .unwrap();

        let features_width = self.features_width();
        let mut x_train = vec![0_f32; features_width * numeric_encoded_dataset.num_train_rows];
        Zip::from(feature_data.columns())
            .and(&self.feature_positions())
            .for_each(|data, position| {
                let column = &self.columns[position.column_position - 1];
                column.preprocess(&data, &mut x_train, features_width, position.row_position);
            });

        let mut x_test = vec![0_f32; features_width * numeric_encoded_dataset.num_test_rows];
        let test_features = ndarray::ArrayView2::from_shape(
            (
                numeric_encoded_dataset.num_test_rows,
                numeric_encoded_dataset.num_features,
            ),
            &numeric_encoded_dataset.x_test,
        )
        .unwrap();
        Zip::from(test_features.columns())
            .and(&self.feature_positions())
            .for_each(|data, position| {
                let column = &self.columns[position.column_position - 1];
                column.preprocess(&data, &mut x_test, features_width, position.row_position);
            });

        self.feature_positions = self.feature_positions();

        Dataset {
            x_train,
            x_test,
            num_distinct_labels: self.num_classes(), // changes after analysis
            ..numeric_encoded_dataset
        }
    }

    // Encodes categorical text values (and all others) into f32 for memory efficiency and type homogenization.
    pub fn numeric_encoded_dataset(&mut self) -> Dataset {
        let mut data = None;
        Spi::connect(|client| {
            // Postgres arrays are 1 indexed and so are SPI tuples...
            let result = client.select(&self.select_sql(), None, None).unwrap();
            let num_rows = result.len();
            let (num_train_rows, num_test_rows) = self.train_test_split(num_rows);
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

                    match &mut column.statistics.categories {
                        // Categorical encoding types
                        Some(categories) => {
                            let key = match column.pg_type.as_str() {
                                "bool" => row[column.position].value::<bool>().unwrap().map(|v| v.to_string()),
                                "int2" => row[column.position].value::<i16>().unwrap().map(|v| v.to_string()),
                                "int4" => row[column.position].value::<i32>().unwrap().map(|v| v.to_string()),
                                "int8" => row[column.position].value::<i64>().unwrap().map(|v| v.to_string()),
                                "float4" => row[column.position].value::<f32>().unwrap().map(|v| v.to_string()),
                                "float8" => row[column.position].value::<f64>().unwrap().map(|v| v.to_string()),
                                "numeric" => row[column.position]
                                    .value::<AnyNumeric>()
                                    .unwrap()
                                    .map(|v| v.to_string()),
                                "bpchar" | "text" | "varchar" => {
                                    row[column.position].value::<String>().unwrap().map(|v| v.to_string())
                                }
                                _ => error!(
                                    "Unhandled type for categorical variable: {} {:?}",
                                    column.name, column.pg_type
                                ),
                            };
                            let key = key.unwrap_or_else(|| NULL_CATEGORY_KEY.to_string());
                            if i < num_train_rows {
                                let len = categories.len();
                                let category = categories.entry(key).or_insert_with_key(|key| {
                                    let value = match key.as_str() {
                                        NULL_CATEGORY_KEY => 0_f32, // NULL values are always Category 0
                                        _ => match &column.preprocessor.encode {
                                            Encode::target | Encode::native | Encode::one_hot { .. } => len as f32,
                                            Encode::ordinal(values) => {
                                                match values.iter().position(|v| v == key.as_str()) {
                                                    Some(i) => (i + 1) as f32,
                                                    None => error!(
                                                        "value is not present in ordinal: {:?}. Valid values: {:?}",
                                                        key, values
                                                    ),
                                                }
                                            }
                                        },
                                    };
                                    Category { value, members: 0 }
                                });
                                category.members += 1;
                                vector.push(category.value);
                            } else {
                                vector.push(column.get_category_value(&key));
                            }
                        }

                        // All quantitative and native types are cast directly to f32
                        None => {
                            if column.array {
                                match column.pg_type.as_str() {
                                    // TODO handle NULL in arrays
                                    "bool[]" => {
                                        let vec = row[column.position].value::<Vec<bool>>().unwrap().unwrap();
                                        check_column_size(column, vec.len());
                                        for j in vec {
                                            vector.push(j as u8 as f32)
                                        }
                                    }
                                    "int2[]" => {
                                        let vec = row[column.position].value::<Vec<i16>>().unwrap().unwrap();
                                        check_column_size(column, vec.len());

                                        for j in vec {
                                            vector.push(j as f32)
                                        }
                                    }
                                    "int4[]" => {
                                        let vec = row[column.position].value::<Vec<i32>>().unwrap().unwrap();
                                        check_column_size(column, vec.len());

                                        for j in vec {
                                            vector.push(j as f32)
                                        }
                                    }
                                    "int8[]" => {
                                        let vec = row[column.position].value::<Vec<i64>>().unwrap().unwrap();
                                        check_column_size(column, vec.len());

                                        for j in vec {
                                            vector.push(j as f32)
                                        }
                                    }
                                    "float4[]" => {
                                        let vec = row[column.position].value::<Vec<f32>>().unwrap().unwrap();
                                        check_column_size(column, vec.len());

                                        for j in vec {
                                            vector.push(j)
                                        }
                                    }
                                    "float8[]" => {
                                        let vec = row[column.position].value::<Vec<f64>>().unwrap().unwrap();
                                        check_column_size(column, vec.len());

                                        for j in vec {
                                            vector.push(j as f32)
                                        }
                                    }
                                    "numeric[]" => {
                                        let vec = row[column.position].value::<Vec<AnyNumeric>>().unwrap().unwrap();
                                        check_column_size(column, vec.len());

                                        for j in vec {
                                            vector.push(j.rescale::<6, 0>().unwrap().try_into().unwrap())
                                        }
                                    }
                                    _ => error!(
                                        "Unhandled type for quantitative array column: {} {:?}",
                                        column.name, column.pg_type
                                    ),
                                }
                            } else {
                                // scalar
                                let float = match column.pg_type.as_str() {
                                    "bool" => row[column.position].value::<bool>().unwrap().map(|v| v as u8 as f32),
                                    "int2" => row[column.position].value::<i16>().unwrap().map(|v| v as f32),
                                    "int4" => row[column.position].value::<i32>().unwrap().map(|v| v as f32),
                                    "int8" => row[column.position].value::<i64>().unwrap().map(|v| v as f32),
                                    "float4" => row[column.position].value::<f32>().unwrap(),
                                    "float8" => row[column.position].value::<f64>().unwrap().map(|v| v as f32),
                                    "numeric" => row[column.position]
                                        .value::<AnyNumeric>()
                                        .unwrap()
                                        .map(|v| v.rescale::<6, 0>().unwrap().try_into().unwrap()),
                                    _ => error!(
                                        "Unhandled type for quantitative scalar column: {} {:?}",
                                        column.name, column.pg_type
                                    ),
                                };
                                match float {
                                    Some(f) => vector.push(f),
                                    None => vector.push(f32::NAN),
                                }
                            }
                        }
                    }
                }
            });

            // recompute the number of features now that we know array widths
            let num_features = self.num_features();
            let num_labels = self.num_labels();

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
                // TODO rename and audit this
                num_distinct_labels: self.num_classes(),
            });

            Ok::<std::option::Option<()>, i64>(Some(())) // this return type is nonsense
        })
        .unwrap();

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

    fn relation_name_quoted(&self) -> String {
        match self.materialized {
            true => self.snapshot_name(), // Snapshot name is already safe.
            false => {
                let (schema_name, table_name) = Self::fully_qualified_table(&self.relation_name);
                format!("\"{}\".\"{}\"", schema_name, table_name)
            }
        }
    }
}

#[inline]
fn check_column_size(column: &mut Column, len: usize) {
    if column.size == 0 {
        column.size = len;
    } else if column.size != len {
        error!(
            "Mismatched array length for feature `{}`. Expected: {} Received: {}",
            column.name, column.size, len
        );
    }
}
