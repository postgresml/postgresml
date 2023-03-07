use std::collections::HashMap;
use std::path::PathBuf;
use once_cell::sync::Lazy;
use pgx::*;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use crate::orm::{Task, TextDataset};

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

pub fn tune(
    task: &Task,
    dataset: TextDataset,
    hyperparams: &JsonB,
) -> PathBuf {
    let task = task.to_string();
    let hyperparams = serde_json::to_string(&hyperparams.0).unwrap();

    let path = Python::with_gil(|py| -> String {
        let tune = PY_MODULE.getattr(py, "tune").unwrap();
        tune
            .call1(py, (&task, &hyperparams, dataset.x_train, dataset.x_test, dataset.y_train, dataset.y_test))
            .unwrap()
            .extract(py)
            .unwrap()
    });
    info!("path: {path:?}");
    PathBuf::from(path)
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
    let json = json.as_object().unwrap();
    let types = json.get("types").unwrap().as_object().unwrap();
    let data = json.get("data").unwrap().as_object().unwrap();
    let column_names = types
        .iter()
        .map(|(name, _type)| name.clone())
        .collect::<Vec<String>>()
        .join(", ");
    let column_types = types
        .iter()
        .map(|(name, type_)| {
            let type_ = match type_.as_str().unwrap() {
                "string" => "TEXT",
                "dict" => "JSONB",
                "int64" => "INT8",
                "int32" => "INT4",
                "int16" => "INT2",
                "float64" => "FLOAT8",
                "float32" => "FLOAT4",
                "float16" => "FLOAT4",
                "bool" => "BOOLEAN",
                _ => error!(
                    "unhandled dataset feature while reading dataset: {:?}",
                    type_
                ),
            };
            format!("{name} {type_}")
        })
        .collect::<Vec<String>>()
        .join(", ");
    let column_placeholders = types
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let placeholder = i + 1;
            format!("${placeholder}")
        })
        .collect::<Vec<String>>()
        .join(", ");
    let num_cols = types.len();
    let num_rows = data.values().next().unwrap().as_array().unwrap().len();
    Spi::run(&format!(r#"DROP TABLE IF EXISTS {table_name}"#)).unwrap();
    Spi::run(&format!(r#"CREATE TABLE {table_name} ({column_types})"#)).unwrap();
    let insert =
        format!(r#"INSERT INTO {table_name} ({column_names}) VALUES ({column_placeholders})"#);
    for i in 0..num_rows {
        let mut row = Vec::with_capacity(num_cols);
        for (name, values) in data {
            let value = values.as_array().unwrap().get(i).unwrap();
            match types.get(name).unwrap().as_str().unwrap() {
                "string" => row.push((
                    PgBuiltInOids::TEXTOID.oid(),
                    value.as_str().unwrap().into_datum(),
                )),
                "dict" => row.push((
                    PgBuiltInOids::JSONBOID.oid(),
                    JsonB(value.clone()).into_datum(),
                )),
                "int64" | "int32" | "int16" => row.push((
                    PgBuiltInOids::INT8OID.oid(),
                    value.as_i64().unwrap().into_datum(),
                )),
                "float64" | "float32" | "float16" => row.push((
                    PgBuiltInOids::FLOAT8OID.oid(),
                    value.as_f64().unwrap().into_datum(),
                )),
                "bool" => row.push((
                    PgBuiltInOids::BOOLOID.oid(),
                    value.as_bool().unwrap().into_datum(),
                )),
                type_ => error!(
                    "unhandled dataset value type while reading dataset: {:?} {:?}",
                    value, type_
                ),
            }
        }
        Spi::run_with_args(&insert, Some(row)).unwrap();
    }

    num_rows
}
