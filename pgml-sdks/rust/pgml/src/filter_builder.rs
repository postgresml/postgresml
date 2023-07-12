use sea_query::{Condition, Expr, IntoCondition, SimpleExpr};

use crate::types::Json;

pub struct FilterBuilder<'a> {
    filter: Json,
    table_name: &'a str,
    column_name: &'a str,
}

impl<'a> FilterBuilder<'a> {
    pub fn new(filter: Json, table_name: &'a str, column_name: &'a str) -> Self {
        Self {
            filter,
            table_name,
            column_name,
        }
    }

    fn build_expression(expression: Expr, filter: serde_json::Value) -> SimpleExpr {
        let (key, value) = filter
            .as_object()
            .expect("Invalid metadata filter configuration")
            .iter()
            .next()
            .expect("Invalid metadata filter configuration");
        let handle_number_value = |value: &serde_json::Value| {
            if value.is_string() {
                Expr::val(value.as_str().unwrap())
            } else if value.is_i64() {
                Expr::val(value.as_i64().unwrap())
            } else if value.is_f64() {
                Expr::val(value.as_f64().unwrap())
            } else {
                panic!("Invalid metadata filter configuration")
            }
        };
        let handle_value = |value: &serde_json::Value| {
            if value.is_string() {
                Expr::val(value.as_str().unwrap())
            } else if value.is_i64() {
                Expr::val(value.as_i64().unwrap())
            } else if value.is_f64() {
                Expr::val(value.as_f64().unwrap())
            } else if value.is_boolean() {
                Expr::val(value.as_bool().unwrap())
            } else {
                panic!("Invalid metadata filter configuration")
            }
        };
        let simple_expression = match key.as_str() {
            "$eq" => expression.eq(handle_value(value)),
            "$ne" => expression.ne(handle_value(value)),
            "$gt" => expression.gt(handle_number_value(value)),
            "$gte" => expression.gte(handle_number_value(value)),
            "$lt" => expression.lt(handle_number_value(value)),
            "$lte" => expression.lte(handle_number_value(value)),
            "$in" => {
                let value = value
                    .as_array()
                    .expect("Invalid metadata filter configuration")
                    .iter()
                    .map(|value| handle_value(value))
                    .collect::<Vec<_>>();
                expression.is_in(value)
            }
            "$nin" => {
                let value = value
                    .as_array()
                    .expect("Invalid metadata filter configuration")
                    .iter()
                    .map(|value| handle_value(value))
                    .collect::<Vec<_>>();
                expression.is_not_in(value)
            }
            _ => panic!("Invalid metadata filter configuration"),
        };
        simple_expression
    }

    fn value_object_is_comparison_operator(value: &serde_json::Value) -> bool {
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
            Self::get_value_type(value)
        } else if value.is_array() {
            let value = &value.as_array().unwrap()[0];
            Self::get_value_type(value)
        } else if value.is_string() {
            "text".to_string()
        } else if value.is_i64() {
            "int".to_string()
        } else if value.is_f64() {
            "float".to_string()
        } else if value.is_boolean() {
            "bool".to_string()
        } else {
            panic!("Invalid metadata filter configuration")
        }
    }

    fn build_recursive(
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
                    "$and" => Self::build_recursive(
                        table_name,
                        column_name,
                        path.clone(),
                        value.clone(),
                        Some(Condition::all()),
                    ),
                    "$or" => Self::build_recursive(
                        table_name,
                        column_name,
                        path.clone(),
                        value.clone(),
                        Some(Condition::any()),
                    ),
                    "$not" => Self::build_recursive(
                        table_name,
                        column_name,
                        path.clone(),
                        value.clone(),
                        Some(Condition::all().not()),
                    ),
                    _ => {
                        local_path.push(key.to_string());
                        if Self::value_object_is_comparison_operator(value) {
                            let ty = Self::get_value_type(value);
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
                            let expression = Self::build_expression(expression, value.clone());
                            expression.into_condition()
                        } else {
                            Self::build_recursive(
                                table_name,
                                column_name,
                                local_path,
                                value.clone(),
                                None,
                            )
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
                    Self::build_recursive(table_name, column_name, local_path, value.clone(), None);
                condition = condition.add(new_condition);
            }
            condition
        } else {
            panic!("Invalid metadata filter configuration")
        }
    }

    pub fn build(self) -> Condition {
        Self::build_recursive(
            self.table_name,
            self.column_name,
            Vec::new(),
            self.filter.0,
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
        FilterBuilder::new(json.into(), "test_table", "metadata")
    }

    #[test]
    fn eq_ne_comparison_operators() {
        let basic_comparison_operators = vec!["=", "<>"];
        let basic_comparison_operators_names = vec!["$eq", "$ne"];
        for (operator, name) in basic_comparison_operators
            .into_iter()
            .zip(basic_comparison_operators_names.into_iter())
        {
            let sql = construct_filter_builder_with_json(json!({
                "id": {name: 1},
                "id2": {"id3": {name: "test"}},
                "id4": {"id5": {"id6": {name: true}}}
            }))
            .build()
            .to_valid_sql_query();
            assert_eq!(
                sql,
                format!(
                    r##"SELECT "id" FROM "test_table" WHERE ("test_table"."metadata"#>>'{{id}}')::int {} 1 AND ("test_table"."metadata"#>>'{{id2,id3}}')::text {} 'test' AND ("test_table"."metadata"#>>'{{id4,id5,id6}}')::bool {} TRUE"##,
                    operator, operator, operator
                )
            );
        }
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
                    r##"SELECT "id" FROM "test_table" WHERE ("test_table"."metadata"#>>'{{id}}')::int {} 1 AND ("test_table"."metadata"#>>'{{id2,id3}}')::int {} 1 "##,
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
                    r##"SELECT "id" FROM "test_table" WHERE ("test_table"."metadata"#>>'{{id}}')::int {} (1) AND ("test_table"."metadata"#>>'{{id2,id3}}')::int {} (1)"##,
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
            r##"SELECT "id" FROM "test_table" WHERE ("test_table"."metadata"#>>'{id}')::int = 1 AND ("test_table"."metadata"#>>'{id2,id3}')::int = 1"##
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
            r##"SELECT "id" FROM "test_table" WHERE ("test_table"."metadata"#>>'{id}')::int = 1 OR ("test_table"."metadata"#>>'{id2,id3}')::int = 1"##
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
            r##"SELECT "id" FROM "test_table" WHERE NOT (("test_table"."metadata"#>>'{id}')::int = 1 AND ("test_table"."metadata"#>>'{id2,id3}')::int = 1)"##
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
            r##"SELECT "id" FROM "test_table" WHERE (("test_table"."metadata"#>>'{id}')::int = 1 OR ("test_table"."metadata"#>>'{id2,id3}')::int = 1) AND ("test_table"."metadata"#>>'{id4}')::int = 1"##
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
            r##"SELECT "id" FROM "test_table" WHERE (("test_table"."metadata"#>>'{id}')::int = 1 AND ("test_table"."metadata"#>>'{id2,id3}')::int = 1) OR ("test_table"."metadata"#>>'{id4}')::int = 1"##
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
            r##"SELECT "id" FROM "test_table" WHERE ("test_table"."metadata"#>>'{metadata,uuid}')::text = '1' OR ("test_table"."metadata"#>>'{metadata,uuid2}')::text = '2'"##
        );
    }
}
