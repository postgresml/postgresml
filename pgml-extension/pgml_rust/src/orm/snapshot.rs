use std::collections::HashMap;

use pgx::*;
use serde_json::json;

use crate::orm::Dataset;
use crate::orm::Sampling;

#[derive(Debug)]
pub struct Snapshot {
    pub id: i64,
    pub relation_name: String,
    pub y_column_name: Vec<String>,
    pub test_size: f32,
    pub test_sampling: Sampling,
    pub status: String,
    pub columns: Option<JsonB>,
    pub analysis: Option<JsonB>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Snapshot {
    pub fn find_last_by_project_id(project_id: i64) -> Option<Snapshot> {
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
                    test_sampling: result.get_datum(5).unwrap(),
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

    pub fn create(
        relation_name: &str,
        y_column_name: &str,
        test_size: f32,
        test_sampling: Sampling,
    ) -> Snapshot {
        let mut snapshot: Option<Snapshot> = None;

        Spi::connect(|client| {
            let result = client.select("INSERT INTO pgml_rust.snapshots (relation_name, y_column_name, test_size, test_sampling, status) VALUES ($1, $2, $3, $4::pgml_rust.sampling, $5) RETURNING id, relation_name, y_column_name, test_size, test_sampling, status, columns, analysis, created_at, updated_at;",
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
                test_sampling: result.get_datum(5).unwrap(),
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

    pub fn dataset(&self) -> Dataset {
        let mut data = None;
        Spi::connect(|client| {
            let json: &serde_json::Value = &self.columns.as_ref().unwrap().0;
            let json = json.as_object().unwrap();
            let feature_columns = json
                .iter()
                .filter_map(|(column, kind)| {
                    let cast = match kind.as_str().unwrap() {
                        "boolean" => "::INT",
                        _ => "",
                    };
                    match self.y_column_name.contains(column) {
                        true => None,
                        false => Some(format!(r#""{}"{}::FLOAT4"#, column, cast)),
                    }
                })
                .collect::<Vec<String>>();
            let label_columns = json
                .iter()
                .filter_map(|(column, kind)| {
                    let cast = match kind.as_str().unwrap() {
                        "boolean" => "::INT",
                        _ => "",
                    };
                    match self.y_column_name.contains(column) {
                        false => None,
                        true => Some(format!(r#""{}"{}::FLOAT4"#, column, cast)),
                    }
                })
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
            data = Some(Dataset {
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
