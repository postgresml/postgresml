use anyhow::Context;
use sea_query::{Expr, Order, SimpleExpr};

pub(crate) struct OrderByBuilder<'a> {
    filter: serde_json::Value,
    table_name: &'a str,
    column_name: &'a str,
}

fn str_to_order(order: &str) -> anyhow::Result<Order> {
    match order {
        "asc" | "ASC" => Ok(Order::Asc),
        "desc" | "DESC" => Ok(Order::Desc),
        _ => anyhow::bail!("Invalid `order_by`. Please refer to examples in the documentation for correct `order_by` syntax"),
    }
}

fn build_recursive_access(key: &str, value: &serde_json::Value) -> anyhow::Result<(String, Order)> {
    if value.is_object() {
        let (new_key, new_value) = value
            .as_object()
            .unwrap()
            .iter()
            .next()
            .context("Invalid `order_by`. Please refer to examples in the documentation for correct `order_by` syntax")?;
        let (path, order) = build_recursive_access(new_key, new_value)?;
        let path = format!("{},{}", key, path);
        Ok((path, order))
    } else if value.is_string() {
        let order = str_to_order(value.as_str().unwrap())?;
        Ok((key.to_string(), order))
    } else {
        Err(anyhow::anyhow!("Invalid `order_by`. Please refer to examples in the documentation for correct `order_by` syntax"))
    }
}

impl<'a> OrderByBuilder<'a> {
    pub fn new(filter: serde_json::Value, table_name: &'a str, column_name: &'a str) -> Self {
        Self {
            filter,
            table_name,
            column_name,
        }
    }

    pub fn build(self) -> anyhow::Result<Vec<(SimpleExpr, Order)>> {
        self.filter
            .as_object()
            .context("`order_by` must be an object")?
            .iter()
            .map(|(k, v)| {
                if k.starts_with("COLUMN_") {
                    Ok((
                        Expr::cust(k.replace("COLUMN_", "")),
                        str_to_order(v.as_str().context("Invalid `order_by`. Please refer to examples in the documentation for correct `order_by` syntax")?)?,
                    ))
                } else if let Ok((path, order)) = build_recursive_access(k, v) {
                    let expr = Expr::cust(format!(
                        "\"{}\".\"{}\"#>'{{{}}}'",
                        self.table_name, self.column_name, path
                    ));
                    Ok((expr, order))
                } else {
                    Err(anyhow::anyhow!("Invalid `order_by`. Please refer to examples in the documentation for correct `order_by` syntax"))
                }
            })
            .collect()
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

    impl ToCustomSqlString for Vec<(SimpleExpr, Order)> {
        fn to_valid_sql_query(self) -> String {
            let mut query = sea_query::Query::select();
            let query = query.column(TestTableIden::Id).from(TestTableIden::Table);
            for (expr, order) in self {
                query.order_by_expr(expr, order);
            }
            query.to_string(PostgresQueryBuilder)
        }
    }

    fn construct_order_by_builder_with_json(json: serde_json::Value) -> OrderByBuilder<'static> {
        OrderByBuilder::new(json, "test_table", "metadata")
    }

    #[test]
    fn test_order_by_builder() {
        let json = json!({
            "id": { "nested_id": "desc"},
            "id_2": "asc"
        });
        let builder = construct_order_by_builder_with_json(json);
        let condition = builder.build().unwrap();
        let expected = r##"SELECT "id" FROM "test_table" ORDER BY "test_table"."metadata"#>'{id,nested_id}' DESC, "test_table"."metadata"#>'{id_2}' ASC"##;
        assert_eq!(condition.to_valid_sql_query(), expected);
    }
}
