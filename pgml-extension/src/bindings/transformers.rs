use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use once_cell::sync::Lazy;
use pgrx::*;
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
    inputs: Vec<&str>,
) -> serde_json::Value {
    crate::bindings::venv::activate();

    let task = serde_json::to_string(task).unwrap();
    let args = serde_json::to_string(args).unwrap();
    let inputs = serde_json::to_string(&inputs).unwrap();

    let results = Python::with_gil(|py| -> String {
        let transform: Py<PyAny> = PY_MODULE.getattr(py, "transform").unwrap().into();

        let result = transform.call1(
            py,
            PyTuple::new(
                py,
                &[task.into_py(py), args.into_py(py), inputs.into_py(py)],
            ),
        );

        let result = match result {
            Err(e) => {
                let traceback = e.traceback(py).unwrap().format().unwrap();
                error!("{traceback} {e}")
            }
            Ok(o) => o.extract(py).unwrap(),
        };

        result
    });
    serde_json::from_str(&results).unwrap()
}

pub fn embed(transformer: &str, inputs: Vec<&str>, kwargs: &serde_json::Value) -> Vec<Vec<f32>> {
    crate::bindings::venv::activate();

    let kwargs = serde_json::to_string(kwargs).unwrap();
    Python::with_gil(|py| -> Vec<Vec<f32>> {
        let embed: Py<PyAny> = PY_MODULE.getattr(py, "embed").unwrap().into();
        let result = embed.call1(
            py,
            PyTuple::new(
                py,
                &[
                    transformer.to_string().into_py(py),
                    inputs.into_py(py),
                    kwargs.into_py(py),
                ],
            ),
        );

        let result = match result {
            Err(e) => {
                let traceback = e.traceback(py).unwrap().format().unwrap();
                error!("{traceback} {e}")
            }
            Ok(o) => o.extract(py).unwrap(),
        };

        result
    })
}

pub fn tune(
    task: &Task,
    dataset: TextDataset,
    hyperparams: &JsonB,
    path: &std::path::PathBuf,
) -> HashMap<String, f64> {
    crate::bindings::venv::activate();

    let task = task.to_string();
    let hyperparams = serde_json::to_string(&hyperparams.0).unwrap();

    let metrics = Python::with_gil(|py| -> HashMap<String, f64> {
        let tune = PY_MODULE.getattr(py, "tune").unwrap();
        let result = tune.call1(
            py,
            (
                &task,
                &hyperparams,
                path.to_str().unwrap(),
                dataset.x_train,
                dataset.x_test,
                dataset.y_train,
                dataset.y_test,
            ),
        );
        let result = match result {
            Err(e) => {
                let traceback = e.traceback(py).unwrap().format().unwrap();
                error!("{traceback} {e}")
            }
            Ok(o) => o,
        };
        result.extract(py).unwrap()
    });
    metrics
}

pub fn generate(model_id: i64, inputs: Vec<&str>, config: JsonB) -> Vec<String> {
    crate::bindings::venv::activate();

    Python::with_gil(|py| -> Vec<String> {
        let generate = PY_MODULE.getattr(py, "generate").unwrap();
        let config = serde_json::to_string(&config.0).unwrap();
        // cloning inputs in case we have to re-call on error is rather unfortunate here
        // similarly, using a json string to pass kwargs is also unfortunate extra parsing
        // it'd be nice to clean all this up one day
        let result = generate.call1(py, (model_id, inputs.clone(), &config));
        let result = match result {
            Err(e) => {
                if e.get_type(py).name().unwrap() == "MissingModelError" {
                    info!("Loading model into cache for connection reuse");
                    let mut dir = std::path::PathBuf::from("/tmp/postgresml/models");
                    dir.push(model_id.to_string());
                    if !dir.exists() {
                        dump_model(model_id, dir.clone());
                    }
                    let task = Spi::get_one_with_args::<String>(
                        "SELECT task::TEXT
                        FROM pgml.projects
                        JOIN pgml.models
                          ON models.project_id = projects.id
                      WHERE models.id = $1",
                        vec![(PgBuiltInOids::INT8OID.oid(), model_id.into_datum())],
                    )
                    .unwrap()
                    .unwrap();

                    let load = PY_MODULE.getattr(py, "load_model").unwrap();
                    let task = Task::from_str(&task).unwrap();
                    load.call1(py, (model_id, task.to_string(), dir)).unwrap();

                    generate.call1(py, (model_id, inputs, config)).unwrap()
                } else {
                    let traceback = e.traceback(py).unwrap().format().unwrap();
                    error!("{traceback} {e}")
                }
            }
            Ok(o) => o,
        };
        result.extract(py).unwrap()
    })
}

fn dump_model(model_id: i64, dir: PathBuf) {
    if dir.exists() {
        std::fs::remove_dir_all(&dir).unwrap();
    }
    std::fs::create_dir_all(&dir).unwrap();
    Spi::connect(|client| {
        let result = client.select("SELECT path, part, data FROM pgml.files WHERE model_id = $1 ORDER BY path ASC, part ASC",
               None,
                Some(vec![
                    (PgBuiltInOids::INT8OID.oid(), model_id.into_datum()),
                ])
            ).unwrap();
        for row in result {
            let mut path = dir.clone();
            path.push(row.get::<String>(1).unwrap().unwrap());
            let data: Vec<u8> = row.get(3).unwrap().unwrap();
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .unwrap();
            file.write(&data).unwrap();
            file.flush().unwrap();
        }
    });
}

pub fn load_dataset(
    name: &str,
    subset: Option<String>,
    limit: Option<usize>,
    kwargs: &serde_json::Value,
) -> usize {
    crate::bindings::venv::activate();

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
                "dict" | "list" => "JSONB",
                "int64" => "INT8",
                "int32" => "INT4",
                "int16" => "INT2",
                "float64" => "FLOAT8",
                "float32" => "FLOAT4",
                "float16" => "FLOAT4",
                "bool" => "BOOLEAN",
                _ => error!("unhandled dataset feature while reading dataset: {}", type_),
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

    // Avoid the existence warning by checking the schema for the table first
    let table_count = Spi::get_one_with_args::<i64>("SELECT COUNT(*) FROM information_schema.tables WHERE table_name = $1 AND table_schema = 'pgml'", vec![
        (PgBuiltInOids::TEXTOID.oid(), table_name.clone().into_datum())
    ]).unwrap().unwrap();
    match table_count {
        1 => Spi::run(&format!(r#"DROP TABLE IF EXISTS {table_name}"#)).unwrap(),
        _ => (),
    }

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
                "dict" | "list" => row.push((
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

pub fn clear_gpu_cache(memory_usage: Option<f32>) -> bool {
    Python::with_gil(|py| -> bool {
        let clear_gpu_cache: Py<PyAny> = PY_MODULE.getattr(py, "clear_gpu_cache").unwrap().into();
        clear_gpu_cache
            .call1(py, PyTuple::new(py, &[memory_usage.into_py(py)]))
            .unwrap()
            .extract(py)
            .unwrap()
    })
}
