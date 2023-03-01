use once_cell::sync::Lazy;
use pgx::*;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

static PY_MODULE: Lazy<Py<PyModule>> = Lazy::new(|| {
    Python::with_gil(|py| -> Py<PyModule> {
        let src = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/bindings/transformers.py"
        ));

        PyModule::from_code(py, src, "", "").unwrap().into()
    })
});

pub fn transform(
    task: &serde_json::Value,
    args: &serde_json::Value,
    inputs: &Vec<String>,
) -> serde_json::Value {
    let task = serde_json::to_string(task).unwrap();
    let args = serde_json::to_string(args).unwrap();
    let inputs = serde_json::to_string(inputs).unwrap();

    let results = Python::with_gil(|py| -> String {
        let transform: Py<PyAny> = PY_MODULE.getattr(py, "transform").unwrap().into();

        transform
            .call1(
                py,
                PyTuple::new(
                    py,
                    &[task.into_py(py), args.into_py(py), inputs.into_py(py)],
                ),
            )
            .unwrap()
            .extract(py)
            .unwrap()
    });
    serde_json::from_str(&results).unwrap()
}

pub fn load_dataset(
    name: &str,
    subset: Option<String>,
    limit: Option<usize>,
    kwargs: &serde_json::Value,
) -> usize {
    let kwargs = serde_json::to_string(kwargs).unwrap();

    let dataset = Python::with_gil(|py| -> String {
        let load_dataset: Py<PyAny> = PY_MODULE.getattr(py, "load_dataset").unwrap().into();
        load_dataset
            .call1(
                py,
                PyTuple::new(
                    py,
                    &[
                        name.into_py(py),
                        subset.into_py(py),
                        limit.into_py(py),
                        kwargs.into_py(py),
                    ],
                ),
            )
            .unwrap()
            .extract(py)
            .unwrap()
    });

    let table_name = format!("pgml.\"{}\"", name);

    // Columns are a (name: String, values: Vec<Value>) pair
    let json: serde_json::Value = serde_json::from_str(&dataset).unwrap();
    let columns = json.as_object().unwrap();
    let column_names = columns
        .iter()
        .map(|(key, _values)| key.clone())
        .collect::<Vec<String>>()
        .join(", ");
    let column_types = columns
        .iter()
        .map(|(key, values)| {
            let mut column = format!("{key} ");
            let first_value = values.as_array().unwrap().first().unwrap();
            if first_value.is_boolean() {
                column.push_str("BOOLEAN");
            } else if first_value.is_i64() {
                column.push_str("INT8");
            } else if first_value.is_f64() {
                column.push_str("FLOAT8");
            } else if first_value.is_string() {
                column.push_str("TEXT");
            } else if first_value.is_object() {
                column.push_str("JSONB");
            } else {
                error!("unhandled pg_type reading dataset: {:?}", first_value);
            };
            column
        })
        .collect::<Vec<String>>()
        .join(", ");

    let num_cols = columns.keys().len();
    let num_rows = columns.values().next().unwrap().as_array().unwrap().len();
    let placeholders = columns
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let placeholder = i + 1;
            format!("${placeholder}")
        })
        .collect::<Vec<String>>()
        .join(", ");
    Spi::run(&format!(r#"DROP TABLE IF EXISTS {table_name}"#)).unwrap();
    Spi::run(&format!(r#"CREATE TABLE {table_name} ({column_types})"#)).unwrap();
    let insert = format!(r#"INSERT INTO {table_name} ({column_names}) VALUES ({placeholders})"#);
    for i in 0..num_rows {
        let mut row = Vec::with_capacity(num_cols);
        for (_column, values) in columns {
            let value = values.as_array().unwrap().get(i).unwrap();
            if value.is_boolean() {
                row.push((
                    PgBuiltInOids::BOOLOID.oid(),
                    value.as_bool().unwrap().into_datum(),
                ));
            } else if value.is_i64() {
                row.push((
                    PgBuiltInOids::INT8OID.oid(),
                    value.as_i64().unwrap().into_datum(),
                ));
            } else if value.is_f64() {
                row.push((
                    PgBuiltInOids::FLOAT8OID.oid(),
                    value.as_f64().unwrap().into_datum(),
                ));
            } else if value.is_string() {
                row.push((
                    PgBuiltInOids::TEXTOID.oid(),
                    value.as_str().unwrap().into_datum(),
                ));
            } else if value.is_object() {
                row.push((
                    PgBuiltInOids::JSONBOID.oid(),
                    JsonB(value.clone()).into_datum(),
                ));
            } else {
                error!("unhandled pg_type reading row: {:?}", value);
            };
        }
        Spi::run_with_args(&insert, Some(row)).unwrap();
    }

    num_rows
}
