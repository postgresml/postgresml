use pgx::*;
use std::str::FromStr;
use std::string::ToString;
use serde_json;
use serde_json::json;
use std::collections::HashMap;

#[derive(PostgresEnum, Copy, Clone, PartialEq, Debug)]
#[allow(non_camel_case_types)]
enum Algorithm {
    linear,
    xgboost,
}

#[derive(PostgresEnum, Copy, Clone, PartialEq, Debug)]
#[allow(non_camel_case_types)]
enum Task {
    regression,
    classification,
}

impl std::str::FromStr for Task {
    type Err = ();

    fn from_str(input: &str) -> Result<Task, Self::Err> {
        match input {
            "regression"     => Ok(Task::regression),
            "classification" => Ok(Task::classification),
            _      => Err(()),
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

#[derive(PostgresEnum, Copy, Clone, PartialEq, Debug)]
#[allow(non_camel_case_types)]
enum Sampling {
    random,
    first,
    last,
}


impl std::str::FromStr for Sampling {
    type Err = ();

    fn from_str(input: &str) -> Result<Sampling, Self::Err> {
        match input {
            "random" => Ok(Sampling::random),
            "first"  => Ok(Sampling::first),
            "last"   => Ok(Sampling::last),
            _        => Err(()),
        }
    }
}

impl std::string::ToString for Sampling {
    fn to_string(&self) -> String {
        match *self {
            Sampling::random => "random".to_string(),
            Sampling::first  => "first".to_string(),
            Sampling::last   => "last".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Project {
    id: i64,
    name: String,
    task: Task,
    created_at: datum::Timestamp,
    updated_at: datum::Timestamp,
}

impl Project {

    fn find(id: i64) -> Project {
        let mut project: Option<Project> = None;

        Spi::connect(|client| {
            let result = client.select("SELECT id, name, task, created_at, updated_at FROM pgml_rust.projects WHERE id = $1 LIMIT 1;",
                Some(1),
                Some(vec![
                    (PgBuiltInOids::INT8OID.oid(), id.into_datum()),
                ])
            ).first();
            project = Some(Project {
                id: result.get_datum(1).unwrap(),
                name: result.get_datum(2).unwrap(),
                task: Task::from_str(result.get_datum(3).unwrap()).unwrap(),
                created_at: result.get_datum(4).unwrap(),
                updated_at: result.get_datum(5).unwrap(),
            });
            Ok(Some(1))
        });
    
        project.unwrap()
    }

    fn find_by_name(name: &str) -> Project {
        let mut project: Option<Project> = None;

        Spi::connect(|client| {
            let result = client.select("SELECT id, name, task, created_at, updated_at FROM pgml_rust.projects WHERE name = $1 LIMIT 1;",
                Some(1),
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), name.into_datum()),
                ])
            ).first();
            project = Some(Project {
                id: result.get_datum(1).unwrap(),
                name: result.get_datum(2).unwrap(),
                task: Task::from_str(result.get_datum(3).unwrap()).unwrap(),
                created_at: result.get_datum(4).unwrap(),
                updated_at: result.get_datum(5).unwrap(),
            });
            Ok(Some(1))
        });
    
        project.unwrap()
    }

    fn create(name: &str, task: Task) -> Project {
        let mut project: Option<Project> = None;

        Spi::connect(|client| {
            let result = client.select("INSERT INTO pgml_rust.projects (name, task) VALUES ($1, $2) RETURNING id, name, task, created_at, updated_at;",
                Some(1),
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), name.into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), task.to_string().into_datum()),
                ])
            ).first();
            project = Some(Project {
                id: result.get_datum(1).unwrap(),
                name: result.get_datum(2).unwrap(),
                task: Task::from_str(result.get_datum(3).unwrap()).unwrap(),
                created_at: result.get_datum(4).unwrap(),
                updated_at: result.get_datum(5).unwrap(),
            });
            Ok(Some(1))
        });
    
        project.unwrap()
    }
}


#[derive(Debug)]
pub struct Snapshot {
    id: i64,
    relation_name: String,
    y_column_name: Vec<String>,
    test_size: f32,
    test_sampling: Sampling,
    columns: Option<serde_json::Value>,
    analysis: Option<serde_json::Value>,
    created_at: datum::Timestamp,
    updated_at: datum::Timestamp,
}

pub struct Columns {
}

pub struct Analysis {
}

impl Snapshot {
    fn create(relation_name: &str, y_column_name: &str, test_size: f32, test_sampling: Sampling) -> Snapshot{
        let mut snapshot: Option<Snapshot> = None;

        Spi::connect(|client| {
            let result = client.select("INSERT INTO pgml_rust.snapshots (relation_name, y_column_name, test_size, test_sampling) VALUES ($1, $2, $3, $4) RETURNING id, relation_name, y_column_name, test_size, test_sampling, columns, analysis, created_at, updated_at;",
                Some(1),
                Some(vec![
                    (PgBuiltInOids::TEXTOID.oid(), relation_name.into_datum()),
                    (PgBuiltInOids::TEXTARRAYOID.oid(), vec![y_column_name].into_datum()),
                    (PgBuiltInOids::FLOAT4OID.oid(), test_size.into_datum()),
                    (PgBuiltInOids::TEXTOID.oid(), test_sampling.to_string().into_datum()),
                ])
            ).first();
            let s = Snapshot {
                id: result.get_datum(1).unwrap(),
                relation_name: result.get_datum(2).unwrap(),
                y_column_name: result.get_datum(3).unwrap(),
                test_size: result.get_datum(4).unwrap(),
                test_sampling: Sampling::from_str(result.get_datum(5).unwrap()).unwrap(),
                columns: match result.get_datum::<datum::Json>(6) {
                    Some(value) => Some(serde_json::from_value(value.0).unwrap()),
                    None => None
                },
                analysis: match result.get_datum::<datum::Json>(7) {
                    Some(value) => Some(serde_json::from_value(value.0).unwrap()),
                    None => None
                },
                created_at: result.get_datum(8).unwrap(),
                updated_at: result.get_datum(9).unwrap(),
            };
            let mut sql = format!(r#"CREATE TABLE "pgml_rust"."snapshot_{}" AS SELECT * FROM {}"#, s.id, s.relation_name);
            if s.test_sampling == Sampling::random {
                sql += " ORDER BY random()";
            }
            client.select(&sql, None, None);
            s.analyze();
            snapshot = Some(s);
            Ok(Some(1))
        });
    
        snapshot.unwrap()
    }

    fn analyze(&self) {
        Spi::connect(|client| {
            let parts = self.relation_name
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
                    error!("Column `{}` not found. Did you pass the correct `y_column_name`?", column)
                }
            }

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
                        stats.push(format!(r#"avg({quoted_column})::FLOAT4 AS "{column}_mean""#));
                        stats.push(format!(r#"stddev({quoted_column})::FLOAT4 AS "{column}_stddev""#));
                        stats.push(format!(r#"percentile_disc(0.25) within group (order by {quoted_column})::FLOAT4 AS "{column}_p25""#));
                        stats.push(format!(r#"percentile_disc(0.5) within group (order by {quoted_column})::FLOAT4 AS "{column}_p50""#));
                        stats.push(format!(r#"percentile_disc(0.75) within group (order by {quoted_column})::FLOAT4 AS "{column}_p75""#));
                        stats.push(format!(r#"count({quoted_column})::FLOAT4 AS "{column}_count""#));
                        stats.push(format!(r#"count(distinct {quoted_column})::FLOAT4 AS "{column}_distinct""#));
                        stats.push(format!(r#"sum(({quoted_column} IS NULL)::INT)::FLOAT4 AS "{column}_nulls""#));
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
                    },
                    &_ => {}
                }
            }            

            let stats = stats.join(",");
            let sql = format!(r#"SELECT {stats} FROM "pgml_rust"."snapshot_{}""#, self.id);
            let result = client.select(&sql, Some(1), None).first();
            let mut analysis = HashMap::new();
            for (i, field) in fields.iter().enumerate() {
                analysis.insert(field, result.get_datum::<f32>((i+1).try_into().unwrap()).unwrap());
            }
            let analysis = pgx::datum::JsonB(json!(&analysis));
            let columns = pgx::datum::JsonB(json!(&columns));
            client.select("UPDATE pgml_rust.snapshots SET analysis = $1, columns = $2 WHERE id = $3", Some(1), Some(vec![
                (PgBuiltInOids::JSONBOID.oid(), analysis.into_datum()),
                (PgBuiltInOids::JSONBOID.oid(), columns.into_datum()),
                (PgBuiltInOids::INT8OID.oid(), self.id.into_datum()),
            ]));

            // TODO set the analysis and columns in memory

            Ok(Some(1))
        });
    }
}

#[pg_extern]
fn create_project(name: &str, task: Task) -> i64 {
    let project = Project::create(name, task);
    info!("{:?}", project);
    project.id
}

// #[pg_extern]
// fn return_table_example() -> impl std::Iterator<Item = (name!(id, Option<i64>), name!(title, Option<String>))> {
//     let tuple = Spi::get_two_with_args("SELECT 1 AS id, 2 AS title;", None, None)
//     vec![tuple].into_iter()
// }

#[pg_extern]
fn create_snapshot(relation_name: &str, y_column_name: &str, test_size: f32, test_sampling: Sampling) -> i64 {
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
