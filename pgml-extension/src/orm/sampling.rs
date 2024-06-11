use pgrx::*;
use serde::Deserialize;

use super::snapshot::Column;

#[derive(PostgresEnum, Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Sampling {
    random,
    last,
    stratified,
}

impl std::str::FromStr for Sampling {
    type Err = ();

    fn from_str(input: &str) -> Result<Sampling, Self::Err> {
        match input {
            "random" => Ok(Sampling::random),
            "last" => Ok(Sampling::last),
            "stratified" => Ok(Sampling::stratified),
            _ => Err(()),
        }
    }
}

impl std::string::ToString for Sampling {
    fn to_string(&self) -> String {
        match *self {
            Sampling::random => "random".to_string(),
            Sampling::last => "last".to_string(),
            Sampling::stratified => "stratified".to_string(),
        }
    }
}

impl Sampling {
    // Implementing the sampling strategy in SQL
    // Effectively orders the table according to the train/test split
    // e.g. first N rows are train, last M rows are test
    // where M is configured by the user
    pub fn get_sql(&self, relation_name: &str, y_column_names: Vec<Column>) -> String {
        let col_string = y_column_names
            .iter()
            .map(|c| c.quoted_name())
            .collect::<Vec<String>>()
            .join(", ");
        match *self {
            Sampling::random => {
                format!("SELECT * FROM {relation_name} ORDER BY RANDOM()")
            }
            Sampling::last => {
                format!("SELECT * FROM {relation_name}")
            }
            Sampling::stratified => {
                format!(
                    "
                    SELECT {col_string}
                    FROM (
                        SELECT 
                            *,
                        ROW_NUMBER() OVER(PARTITION BY {col_string} ORDER BY RANDOM()) AS rn
                        FROM {relation_name}
                    ) AS subquery
                    ORDER BY rn, RANDOM();
                "
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::orm::snapshot::{Preprocessor, Statistics};

    use super::*;

    fn get_column_fixtures() -> Vec<Column> {
        vec![
            Column {
                name: "col1".to_string(),
                pg_type: "text".to_string(),
                nullable: false,
                label: true,
                position: 0,
                size: 0,
                array: false,
                preprocessor: Preprocessor::default(),
                statistics: Statistics::default(),
            },
            Column {
                name: "col2".to_string(),
                pg_type: "text".to_string(),
                nullable: false,
                label: true,
                position: 0,
                size: 0,
                array: false,
                preprocessor: Preprocessor::default(),
                statistics: Statistics::default(),
            },
        ]
    }

    #[test]
    fn test_get_sql_random_sampling() {
        let sampling = Sampling::random;
        let columns = get_column_fixtures();
        let sql = sampling.get_sql("my_table", columns);
        assert_eq!(sql, "SELECT * FROM my_table ORDER BY RANDOM()");
    }

    #[test]
    fn test_get_sql_last_sampling() {
        let sampling = Sampling::last;
        let columns = get_column_fixtures();
        let sql = sampling.get_sql("my_table", columns);
        assert_eq!(sql, "SELECT * FROM my_table");
    }

    #[test]
    fn test_get_sql_stratified_sampling() {
        let sampling = Sampling::stratified;
        let columns = get_column_fixtures();
        let sql = sampling.get_sql("my_table", columns);
        let expected_sql = "
                    SELECT \"col1\", \"col2\"
                    FROM (
                        SELECT 
                            *,
                        ROW_NUMBER() OVER(PARTITION BY \"col1\", \"col2\" ORDER BY RANDOM()) AS rn
                        FROM my_table
                    ) AS subquery
                    ORDER BY rn, RANDOM();
                ";
        assert_eq!(sql, expected_sql);
    }
}
