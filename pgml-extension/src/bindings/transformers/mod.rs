use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{anyhow, bail, Context, Result};
use lazy_static::*;
use pgrx::*;
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use serde_json::Value;
use std::cmp::Ordering;
use std::env;
use std::sync::RwLock;

use crate::config::get_config;
use crate::create_pymodule;
use crate::orm::guc;
use crate::orm::{Task, TextDataset};

use super::TracebackError;

pub mod whitelist;

create_pymodule!("/src/bindings/transformers/transformers.py");

lazy_static! {
    // Record the previous applied ENVs.
    static ref ENVS_APPLIED: RwLock<Box<BTreeMap<&'static str, String>>> =
        RwLock::new(Box::new(BTreeMap::new()));
}

pub fn transform(
    task: &serde_json::Value,
    args: &serde_json::Value,
    inputs: Vec<&str>,
) -> Result<serde_json::Value> {
    set_env();
    crate::bindings::python::activate()?;

    whitelist::verify_task(task)?;

    let task = serde_json::to_string(task)?;
    let args = serde_json::to_string(args)?;
    let inputs = serde_json::to_string(&inputs)?;

    let results = Python::with_gil(|py| -> Result<String> {
        let transform: Py<PyAny> = get_module!(PY_MODULE)
            .getattr(py, "transform")
            .format_traceback(py)?;

        let output = transform
            .call1(
                py,
                PyTuple::new(
                    py,
                    &[task.into_py(py), args.into_py(py), inputs.into_py(py)],
                ),
            )
            .format_traceback(py)?;

        output.extract(py).format_traceback(py)
    })?;

    Ok(serde_json::from_str(&results)?)
}

pub fn get_model_from(task: &Value) -> Result<String> {
    Python::with_gil(|py| -> Result<String> {
        let get_model_from = get_module!(PY_MODULE)
            .getattr(py, "get_model_from")
            .format_traceback(py)?;
        let model = get_model_from
            .call1(py, PyTuple::new(py, &[task.to_string().into_py(py)]))
            .format_traceback(py)?;
        model.extract(py).format_traceback(py)
    })
}

pub fn embed(
    transformer: &str,
    inputs: Vec<&str>,
    kwargs: &serde_json::Value,
) -> Result<Vec<Vec<f32>>> {
    set_env();
    crate::bindings::python::activate()?;

    let kwargs = serde_json::to_string(kwargs)?;
    Python::with_gil(|py| -> Result<Vec<Vec<f32>>> {
        let embed: Py<PyAny> = get_module!(PY_MODULE)
            .getattr(py, "embed")
            .format_traceback(py)?;
        let output = embed
            .call1(
                py,
                PyTuple::new(
                    py,
                    &[
                        transformer.to_string().into_py(py),
                        inputs.into_py(py),
                        kwargs.into_py(py),
                    ],
                ),
            )
            .format_traceback(py)?;

        output.extract(py).format_traceback(py)
    })
}

pub fn tune(
    task: &Task,
    dataset: TextDataset,
    hyperparams: &JsonB,
    path: &Path,
) -> Result<HashMap<String, f64>> {
    set_env();
    crate::bindings::python::activate()?;

    let task = task.to_string();
    let hyperparams = serde_json::to_string(&hyperparams.0)?;

    Python::with_gil(|py| -> Result<HashMap<String, f64>> {
        let tune = get_module!(PY_MODULE)
            .getattr(py, "tune")
            .format_traceback(py)?;
        let path = path.to_string_lossy();
        let output = tune
            .call1(
                py,
                (
                    &task,
                    &hyperparams,
                    path.as_ref(),
                    dataset.x_train,
                    dataset.x_test,
                    dataset.y_train,
                    dataset.y_test,
                ),
            )
            .format_traceback(py)?;

        output.extract(py).format_traceback(py)
    })
}

pub fn generate(model_id: i64, inputs: Vec<&str>, config: JsonB) -> Result<Vec<String>> {
    set_env();
    crate::bindings::python::activate()?;

    Python::with_gil(|py| -> Result<Vec<String>> {
        let generate = get_module!(PY_MODULE)
            .getattr(py, "generate")
            .format_traceback(py)?;
        let config = serde_json::to_string(&config.0)?;
        // cloning inputs in case we have to re-call on error is rather unfortunate here
        // similarly, using a json string to pass kwargs is also unfortunate extra parsing
        // it'd be nice to clean all this up one day
        let result = generate.call1(py, (model_id, inputs.clone(), &config));
        let result = match result {
            Err(e) => {
                if e.get_type(py).name()? == "MissingModelError" {
                    info!("Loading model into cache for connection reuse");
                    let mut dir = std::path::PathBuf::from("/tmp/postgresml/models");
                    dir.push(model_id.to_string());
                    if !dir.exists() {
                        dump_model(model_id, dir.clone())?;
                    }
                    let task = Spi::get_one_with_args::<String>(
                        "SELECT task::TEXT
                        FROM pgml.projects
                        JOIN pgml.models
                          ON models.project_id = projects.id
                      WHERE models.id = $1",
                        vec![(PgBuiltInOids::INT8OID.oid(), model_id.into_datum())],
                    )?
                    .ok_or(anyhow!("task query returned None"))?;

                    let load = get_module!(PY_MODULE).getattr(py, "load_model")?;
                    let task = Task::from_str(&task)
                        .map_err(|_| anyhow!("could not make a Task from {task}"))?;
                    load.call1(py, (model_id, task.to_string(), dir))
                        .format_traceback(py)?;

                    generate
                        .call1(py, (model_id, inputs, config))
                        .format_traceback(py)?
                } else {
                    return Err(e.into());
                }
            }
            Ok(o) => o,
        };
        result.extract(py).format_traceback(py)
    })
}

fn dump_model(model_id: i64, dir: PathBuf) -> Result<()> {
    if dir.exists() {
        std::fs::remove_dir_all(&dir).context("failed to remove directory while dumping model")?;
    }
    std::fs::create_dir_all(&dir).context("failed to create directory while dumping model")?;
    Spi::connect(|client| -> Result<()> {
        let result = client.select("SELECT path, part, data FROM pgml.files WHERE model_id = $1 ORDER BY path ASC, part ASC",
               None,
                Some(vec![
                    (PgBuiltInOids::INT8OID.oid(), model_id.into_datum()),
                ])
            )?;
        for row in result {
            let mut path = dir.clone();
            path.push(
                row.get::<String>(1)?
                    .ok_or(anyhow!("row get ordinal 1 returned None"))?,
            );
            let data: Vec<u8> = row
                .get(3)?
                .ok_or(anyhow!("row get ordinal 3 returned None"))?;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;

            let _num_bytes = file.write(&data)?;
            file.flush()?;
        }
        Ok(())
    })
}

pub fn load_dataset(
    name: &str,
    subset: Option<String>,
    limit: Option<usize>,
    kwargs: &serde_json::Value,
) -> Result<usize> {
    set_env();
    crate::bindings::python::activate()?;

    let kwargs = serde_json::to_string(kwargs)?;

    let dataset = Python::with_gil(|py| -> Result<String> {
        let load_dataset: Py<PyAny> = get_module!(PY_MODULE)
            .getattr(py, "load_dataset")
            .format_traceback(py)?;
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
            .format_traceback(py)?
            .extract(py)
            .format_traceback(py)
    })?;

    let table_name = format!("pgml.\"{}\"", name);

    // Columns are a (name: String, values: Vec<Value>) pair
    let json: serde_json::Value = serde_json::from_str(&dataset)?;
    let json = json
        .as_object()
        .ok_or(anyhow!("dataset json is not object"))?;
    let types = json
        .get("types")
        .ok_or(anyhow!("dataset json missing `types` key"))?
        .as_object()
        .ok_or(anyhow!("dataset `types` key is not an object"))?;
    let data = json
        .get("data")
        .ok_or(anyhow!("dataset json missing `data` key"))?
        .as_object()
        .ok_or(anyhow!("dataset `data` key is not an object"))?;
    let column_names = types
        .iter()
        .map(|(name, _type)| name.clone())
        .collect::<Vec<String>>()
        .join(", ");
    let column_types = types
        .iter()
        .map(|(name, type_)| -> Result<String> {
            let type_ = type_
                .as_str()
                .ok_or(anyhow!("expected {type_} to be a json string"))?;
            let type_ = match type_ {
                "string" => "TEXT",
                "dict" | "list" => "JSONB",
                "int64" => "INT8",
                "int32" => "INT4",
                "int16" => "INT2",
                "float64" => "FLOAT8",
                "float32" => "FLOAT4",
                "float16" => "FLOAT4",
                "bool" => "BOOLEAN",
                _ => bail!("unhandled dataset feature while reading dataset: {type_}"),
            };
            Ok(format!("{name} {type_}"))
        })
        .collect::<Result<Vec<String>>>()?
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
    let num_rows = data
        .values()
        .next()
        .ok_or(anyhow!("dataset json has no fields"))?
        .as_array()
        .ok_or(anyhow!("dataset json field is not an array"))?
        .len();

    // Avoid the existence warning by checking the schema for the table first
    let table_count = Spi::get_one_with_args::<i64>("SELECT COUNT(*) FROM information_schema.tables WHERE table_name = $1 AND table_schema = 'pgml'", vec![
        (PgBuiltInOids::TEXTOID.oid(), table_name.clone().into_datum())
    ])?.ok_or(anyhow!("table count query returned None"))?;
    if table_count == 1 {
        Spi::run(&format!(r#"DROP TABLE IF EXISTS {table_name}"#))?;
    }

    Spi::run(&format!(r#"CREATE TABLE {table_name} ({column_types})"#))?;
    let insert =
        format!(r#"INSERT INTO {table_name} ({column_names}) VALUES ({column_placeholders})"#);
    for i in 0..num_rows {
        let mut row = Vec::with_capacity(num_cols);
        for (name, values) in data {
            let value = values
                .as_array()
                .ok_or_else(|| anyhow!("expected {values} to be an array"))?
                .get(i)
                .ok_or_else(|| anyhow!("invalid index {i} for {values}"))?;
            match types
                .get(name)
                .ok_or_else(|| anyhow!("{types:?} expected to have key {name}"))?
                .as_str()
                .ok_or_else(|| anyhow!("json field {name} expected to be string"))?
            {
                "string" => row.push((
                    PgBuiltInOids::TEXTOID.oid(),
                    value
                        .as_str()
                        .ok_or_else(|| anyhow!("expected {value} to be string"))?
                        .into_datum(),
                )),
                "dict" | "list" => row.push((
                    PgBuiltInOids::JSONBOID.oid(),
                    JsonB(value.clone()).into_datum(),
                )),
                "int64" | "int32" | "int16" => row.push((
                    PgBuiltInOids::INT8OID.oid(),
                    value
                        .as_i64()
                        .ok_or_else(|| anyhow!("expected {value} to be i64"))?
                        .into_datum(),
                )),
                "float64" | "float32" | "float16" => row.push((
                    PgBuiltInOids::FLOAT8OID.oid(),
                    value
                        .as_f64()
                        .ok_or_else(|| anyhow!("expected {value} to be f64"))?
                        .into_datum(),
                )),
                "bool" => row.push((
                    PgBuiltInOids::BOOLOID.oid(),
                    value
                        .as_bool()
                        .ok_or_else(|| anyhow!("expected {value} to be bool"))?
                        .into_datum(),
                )),
                type_ => {
                    bail!("unhandled dataset value type while reading dataset: {value:?} {type_:?}")
                }
            }
        }
        Spi::run_with_args(&insert, Some(row))?
    }

    Ok(num_rows)
}

pub fn clear_gpu_cache(memory_usage: Option<f32>) -> Result<bool> {
    set_env();
    crate::bindings::python::activate().unwrap();

    Python::with_gil(|py| -> Result<bool> {
        let clear_gpu_cache: Py<PyAny> = get_module!(PY_MODULE)
            .getattr(py, "clear_gpu_cache")
            .format_traceback(py)?;
        let success = clear_gpu_cache
            .call1(py, PyTuple::new(py, &[memory_usage.into_py(py)]))
            .format_traceback(py)?
            .extract(py)
            .format_traceback(py)?;
        Ok(success)
    })
}

// Called before hugginface python APIs. Setup ENVs for HuggingFace. See
// https://huggingface.co/docs/huggingface_hub/package_reference/environment_variables#hfhuboffline
pub fn set_env() {
    let envs_to_apply = guc::gen_hf_env_map();
    let py_inited = unsafe { ffi::Py_IsInitialized() != 0 };

    {
        // This block can not be removed. It's used to drop read lock
        // read lock held
        let envs_current = ENVS_APPLIED.read().unwrap();

        if py_inited {
            if envs_to_apply.cmp(&envs_current) != Ordering::Equal {
                // Python had been initialized and GUCs changed. Report warning and do nothing.
                warning!("HuggingFace env changed in this session. Please start a new session with new GUC values.");
                return;
            } else {
                // GUCs haven't been changed. Just return.
                return;
            }
        }
        // read lock dropped
    }

    // Set the env
    for (k, v) in &envs_to_apply {
        if v.trim().is_empty() {
            env::remove_var(k);
        } else {
            env::set_var(k, v);
        }
    }
    // Record current ENVs
    // write lock held
    let _ = std::mem::replace(ENVS_APPLIED.write().unwrap().as_mut(), envs_to_apply);
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;

    #[pg_test]
    fn test_set_env() {
        use crate::config::set_config;

        let tmp_path: &str = "/tmp/pgml";

        set_config(guc::CONFIG_CACHE, tmp_path).unwrap();

        set_env();
        let _ = crate::bindings::python::activate();

        let base_path: PathBuf;

        match get_config(guc::CONFIG_CACHE) {
            Some(value) => {
                base_path = PathBuf::from(value);
                let base_path = base_path.display();

                assert_eq!(
                    env::var("HF_HOME").unwrap(),
                    format!("{}/huggingface", base_path)
                );
                assert_eq!(
                    env::var("SENTENCE_TRANSFORMERS_HOME").unwrap(),
                    format!("{}/torch", base_path)
                );
                assert_eq!(
                    env::var("HF_HOME").unwrap(),
                    format!("{}/huggingface", tmp_path)
                );
                assert_eq!(
                    env::var("SENTENCE_TRANSFORMERS_HOME").unwrap(),
                    format!("{}/torch", tmp_path)
                );
            }
            None => {
                assert!(env::var("HF_HOME").is_err());
                assert!(env::var("SENTENCE_TRANSFORMERS_HOME").is_err());
            }
        }
    }
}
