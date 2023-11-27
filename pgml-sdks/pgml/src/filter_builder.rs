use sea_query::{
    extension::postgres::PgExpr, value::ArrayType, Condition, Expr, IntoCondition, SimpleExpr,
};

fn get_sea_query_array_type(value: &serde_json::Value) -> ArrayType {
    if value.is_null() {
        panic!("Invalid metadata filter configuration")
    } else if value.is_string() {
        ArrayType::String
    } else if value.is_i64() || value.is_u64() {
        ArrayType::BigInt
    } else if value.is_f64() {
        ArrayType::Double
    } else if value.is_boolean() {
        ArrayType::Bool
    } else if value.is_array() {
        let value = value
            .as_array()
            .expect("Invalid metadata filter configuration");
        get_sea_query_array_type(&value[0])
    } else {
        panic!("Invalid metadata filter configuration")
    }
}

fn serde_value_to_sea_query_value(value: &serde_json::Value) -> sea_query::Value {
    if value.is_string() {
        sea_query::Value::String(Some(Box::new(value.as_str().unwrap().to_string())))
    } else if value.is_i64() {
        sea_query::Value::BigInt(Some(value.as_i64().unwrap()))
    } else if value.is_f64() {
        sea_query::Value::Double(Some(value.as_f64().unwrap()))
    } else if value.is_boolean() {
        sea_query::Value::Bool(Some(value.as_bool().unwrap()))
    } else if value.is_array() {
        let value = value.as_array().unwrap();
        let ty = get_sea_query_array_type(&value[0]);
        let value = Some(Box::new(
            value.iter().map(serde_value_to_sea_query_value).collect(),
        ));
        sea_query::Value::Array(ty, value)
    } else if value.is_object() {
        sea_query::Value::Json(Some(Box::new(value.clone())))
    } else {
        panic!("Invalid metadata filter configuration")
    }
}

fn reconstruct_json(path: Vec<String>, value: serde_json::Value) -> serde_json::Value {
    if path.is_empty() {
        value
    } else {
        let mut object = serde_json::Map::new();
        object.insert(path[0].clone(), reconstruct_json(path[1..].to_vec(), value));
        serde_json::Value::Object(object)
    }
}

fn build_expression(expression: Expr, filter: serde_json::Value) -> SimpleExpr {
    let (key, value) = filter
        .as_object()
        .expect("Invalid metadata filter configuration")
        .iter()
        .next()
        .expect("Invalid metadata filter configuration");
    let simple_expression = match key.as_str() {
        "$gt" => expression.gt(Expr::val(serde_value_to_sea_query_value(value))),
        "$gte" => expression.gte(Expr::val(serde_value_to_sea_query_value(value))),
        "$lt" => expression.lt(Expr::val(serde_value_to_sea_query_value(value))),
        "$lte" => expression.lte(Expr::val(serde_value_to_sea_query_value(value))),
        "$in" => {
            let value = value
                .as_array()
                .expect("Invalid metadata filter configuration")
                .iter()
                // .map(|value| handle_value(value))
                .map(|value| Expr::val(serde_value_to_sea_query_value(value)))
                .collect::<Vec<_>>();
            expression.is_in(value)
        }
        "$nin" => {
            let value = value
                .as_array()
                .expect("Invalid metadata filter configuration")
                .iter()
                .map(|value| Expr::val(serde_value_to_sea_query_value(value)))
                .collect::<Vec<_>>();
            expression.is_not_in(value)
        }
        _ => panic!("Invalid metadata filter configuration"),
    };
    simple_expression
}

fn value_is_object_and_is_comparison_operator(value: &serde_json::Value) -> bool {
    value.is_object()
        && value.as_object().unwrap().iter().any(|(key, _)| {
            matches!(
                key.as_str(),
                "$eq" | "$ne" | "$gt" | "$gte" | "$lt" | "$lte" | "$in" | "$nin"
            )
        })
}

fn get_value_type(value: &serde_json::Value) -> String {
    if value.is_object() {
        let (_, value) = value
            .as_object()
            .expect("Invalid metadata filter configuration")
            .iter()
            .next()
            .unwrap();
        get_value_type(value)
    } else if value.is_array() {
        let value = &value.as_array().unwrap()[0];
        get_value_type(value)
    } else if value.is_string() {
        "text".to_string()
    } else if value.is_i64() || value.is_f64() {
        "float8".to_string()
    } else if value.is_boolean() {
        "bool".to_string()
    } else {
        panic!("Invalid metadata filter configuration")
    }
}

fn build_recursive<'a>(
    table_name: &'a str,
    column_name: &'a str,
    path: Vec<String>,
    filter: serde_json::Value,
    condition: Option<Condition>,
) -> Condition {
    if filter.is_object() {
        let mut condition = condition.unwrap_or(Condition::all());
        for (key, value) in filter.as_object().unwrap() {
            let mut local_path = path.clone();
            let sub_condition = match key.as_str() {
                "$and" => build_recursive(
                    table_name,
                    column_name,
                    local_path,
                    value.clone(),
                    Some(Condition::all()),
                ),
                "$or" => build_recursive(
                    table_name,
                    column_name,
                    local_path,
                    value.clone(),
                    Some(Condition::any()),
                ),
                "$not" => build_recursive(
                    table_name,
                    column_name,
                    local_path,
                    value.clone(),
                    Some(Condition::all().not()),
                ),
                _ => {
                    local_path.push(key.to_string());
                    if value_is_object_and_is_comparison_operator(value) {
                        let (operator, final_value) =
                            value.as_object().unwrap().iter().next().unwrap();
                        // If we are checking whether two values are equal or not equal, we need to reconstruct the json so we
                        // can use the contains operator
                        let expression = if operator == "$eq" || operator == "$ne" {
                            let json = reconstruct_json(local_path, final_value.to_owned());
                            let expression = Expr::cust(
                                format!("\"{}\".\"{}\"", table_name, column_name).as_str(),
                            );
                            let expression = Expr::expr(expression);
                            if operator == "$eq" {
                                expression
                                    .contains(Expr::val(serde_value_to_sea_query_value(&json)))
                            } else {
                                expression
                                    .not()
                                    .contains(Expr::val(serde_value_to_sea_query_value(&json)))
                            }
                        } else {
                            // If we are not checking whether two values are equal or not equal, we need to cast it to the correct type before doing the comparison
                            let ty = get_value_type(value);
                            let expression = Expr::cust(
                                format!(
                                    "(\"{}\".\"{}\"#>>'{{{}}}')::{}",
                                    table_name,
                                    column_name,
                                    local_path.join(","),
                                    ty
                                )
                                .as_str(),
                            );
                            let expression = Expr::expr(expression);
                            build_expression(expression, value.clone())
                        };
                        expression.into_condition()
                    } else {
                        build_recursive(table_name, column_name, local_path, value.clone(), None)
                    }
                }
            };
            condition = condition.add(sub_condition);
        }
        condition
    } else if filter.is_array() {
        let mut condition = condition.expect("Invalid metadata filter configuration");
        for value in filter.as_array().unwrap() {
            let local_path = path.clone();
            let new_condition =
                build_recursive(table_name, column_name, local_path, value.clone(), None);
            condition = condition.add(new_condition);
        }
        condition
    } else {
        panic!("Invalid metadata filter configuration")
    }
}

pub struct FilterBuilder<'a> {
    filter: serde_json::Value,
    table_name: &'a str,
    column_name: &'a str,
}

impl<'a> FilterBuilder<'a> {
    pub fn new(filter: serde_json::Value, table_name: &'a str, column_name: &'a str) -> Self {
        Self {
            filter,
            table_name,
            column_name,
        }
    }

    pub fn build(self) -> Condition {
        build_recursive(
            self.table_name,
            self.column_name,
            Vec::new(),
            self.filter,
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_query::{enum_def, PostgresQueryBuilder};
    use serde_json::json;

    #[enum_def]
    #[allow(unused)]
    struct TestTable {
        id: i64,
    }

    trait ToCustomSqlString {
        fn to_valid_sql_query(self) -> String;
    }

    impl ToCustomSqlString for Condition {
        fn to_valid_sql_query(self) -> String {
            let mut query = sea_query::Query::select();
            let query = query
                .column(TestTableIden::Id)
                .from(TestTableIden::Table)
                .cond_where(self);
            query.to_string(PostgresQueryBuilder)
        }
    }

    fn construct_filter_builder_with_json(json: serde_json::Value) -> FilterBuilder<'static> {
        FilterBuilder::new(json, "test_table", "metadata")
    }

    #[test]
    fn eq_operator() {
        let sql = construct_filter_builder_with_json(json!({
            "id": {"$eq": 1},
            "id2": {"id3": {"$eq": "test"}},
            "id4": {"id5": {"id6": {"$eq": true}}},
            "id7": {"id8": {"id9": {"id10": {"$eq": [1, 2, 3]}}}}
        }))
        .build()
        .to_valid_sql_query();
        assert_eq!(
            sql,
            r#"SELECT "id" FROM "test_table" WHERE "test_table"."metadata" @> E'{\"id\":1}' AND "test_table"."metadata" @> E'{\"id2\":{\"id3\":\"test\"}}' AND "test_table"."metadata" @> E'{\"id4\":{\"id5\":{\"id6\":true}}}' AND "test_table"."metadata" @> E'{\"id7\":{\"id8\":{\"id9\":{\"id10\":[1,2,3]}}}}'"#
        );
    }

    #[test]
    fn ne_operator() {
        let sql = construct_filter_builder_with_json(json!({
            "id": {"$ne": 1},
            "id2": {"id3": {"$ne": "test"}},
            "id4": {"id5": {"id6": {"$ne": true}}},
            "id7": {"id8": {"id9": {"id10": {"$ne": [1, 2, 3]}}}}
        }))
        .build()
        .to_valid_sql_query();
        assert_eq!(
            sql,
            r#"SELECT "id" FROM "test_table" WHERE NOT "test_table"."metadata" @> E'{\"id\":1}' AND NOT "test_table"."metadata" @> E'{\"id2\":{\"id3\":\"test\"}}' AND NOT "test_table"."metadata" @> E'{\"id4\":{\"id5\":{\"id6\":true}}}' AND NOT "test_table"."metadata" @> E'{\"id7\":{\"id8\":{\"id9\":{\"id10\":[1,2,3]}}}}'"#
        );
    }

    #[test]
    fn numeric_comparison_operators() {
        let basic_comparison_operators = vec![">", ">=", "<", "<="];
        let basic_comparison_operators_names = vec!["$gt", "$gte", "$lt", "$lte"];
        for (operator, name) in basic_comparison_operators
            .into_iter()
            .zip(basic_comparison_operators_names.into_iter())
        {
            let sql = construct_filter_builder_with_json(json!({
                "id": {name: 1},
                "id2": {"id3": {name: 1}}
            }))
            .build()
            .to_valid_sql_query();
            assert_eq!(
                sql,
                format!(
                    r##"SELECT "id" FROM "test_table" WHERE ("test_table"."metadata"#>>'{{id}}')::float8 {} 1 AND ("test_table"."metadata"#>>'{{id2,id3}}')::float8 {} 1"##,
                    operator, operator
                )
            );
        }
    }

    #[test]
    fn array_comparison_operators() {
        let array_comparison_operators = vec!["IN", "NOT IN"];
        let array_comparison_operators_names = vec!["$in", "$nin"];
        for (operator, name) in array_comparison_operators
            .into_iter()
            .zip(array_comparison_operators_names.into_iter())
        {
            let sql = construct_filter_builder_with_json(json!({
                "id": {name: [1]},
                "id2": {"id3": {name: [1]}}
            }))
            .build()
            .to_valid_sql_query();
            assert_eq!(
                sql,
                format!(
                    r##"SELECT "id" FROM "test_table" WHERE ("test_table"."metadata"#>>'{{id}}')::float8 {} (1) AND ("test_table"."metadata"#>>'{{id2,id3}}')::float8 {} (1)"##,
                    operator, operator
                )
            );
        }
    }

    #[test]
    fn and_operator() {
        let sql = construct_filter_builder_with_json(json!({
            "$and": [
                {"id": {"$eq": 1}},
                {"id2": {"id3": {"$eq": 1}}}
            ]
        }))
        .build()
        .to_valid_sql_query();
        assert_eq!(
            sql,
            r#"SELECT "id" FROM "test_table" WHERE "test_table"."metadata" @> E'{\"id\":1}' AND "test_table"."metadata" @> E'{\"id2\":{\"id3\":1}}'"#
        );
    }

    #[test]
    fn or_operator() {
        let sql = construct_filter_builder_with_json(json!({
            "$or": [
                {"id": {"$eq": 1}},
                {"id2": {"id3": {"$eq": 1}}}
            ]
        }))
        .build()
        .to_valid_sql_query();
        assert_eq!(
            sql,
            r#"SELECT "id" FROM "test_table" WHERE "test_table"."metadata" @> E'{\"id\":1}' OR "test_table"."metadata" @> E'{\"id2\":{\"id3\":1}}'"#
        );
    }

    #[test]
    fn not_operator() {
        let sql = construct_filter_builder_with_json(json!({
        "$not": [
                {"id": {"$eq": 1}},
                {"id2": {"id3": {"$eq": 1}}}
            ]
        }))
        .build()
        .to_valid_sql_query();
        assert_eq!(
            sql,
            r#"SELECT "id" FROM "test_table" WHERE NOT ("test_table"."metadata" @> E'{\"id\":1}' AND "test_table"."metadata" @> E'{\"id2\":{\"id3\":1}}')"#
        );
    }

    #[test]
    fn random_difficult_tests() {
        let sql = construct_filter_builder_with_json(json!({
            "$and": [
                {"$or": [
                        {"id": {"$eq": 1}},
                        {"id2": {"id3": {"$eq": 1}}}
                    ]
                },
                {"id4": {"$eq": 1}}
            ]
        }))
        .build()
        .to_valid_sql_query();
        assert_eq!(
            sql,
            r#"SELECT "id" FROM "test_table" WHERE ("test_table"."metadata" @> E'{\"id\":1}' OR "test_table"."metadata" @> E'{\"id2\":{\"id3\":1}}') AND "test_table"."metadata" @> E'{\"id4\":1}'"#
        );
        let sql = construct_filter_builder_with_json(json!({
            "$or": [
                {"$and": [
                        {"id": {"$eq": 1}},
                        {"id2": {"id3": {"$eq": 1}}}
                    ]
                },
                {"id4": {"$eq": 1}}
            ]
        }))
        .build()
        .to_valid_sql_query();
        assert_eq!(
            sql,
            r#"SELECT "id" FROM "test_table" WHERE ("test_table"."metadata" @> E'{\"id\":1}' AND "test_table"."metadata" @> E'{\"id2\":{\"id3\":1}}') OR "test_table"."metadata" @> E'{\"id4\":1}'"#
        );
        let sql = construct_filter_builder_with_json(json!({
            "metadata": {"$or": [
                {"uuid": {"$eq": "1"}},
                {"uuid2": {"$eq": "2"}}
            ]}
        }))
        .build()
        .to_valid_sql_query();
        assert_eq!(
            sql,
            r#"SELECT "id" FROM "test_table" WHERE "test_table"."metadata" @> E'{\"metadata\":{\"uuid\":\"1\"}}' OR "test_table"."metadata" @> E'{\"metadata\":{\"uuid2\":\"2\"}}'"#
        );
    }
}
