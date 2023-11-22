//! Embedding models
//!
//! Currently uses Python packages `InstructorEmbedding`, `sentence_transformers`, and `transformers`.
//! All models starting with 'hkunlp/instructor' use `InstructorEmbedding`, 'sentence-transformers' use
//! `SentenceTransformers`, and the rest use `transformers.AutoModel`.
//!
//! The models are cached to prevent loading on each `embed` invocation.

use std::collections::HashMap;

use anyhow::{anyhow, bail, Context, Result};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use pyo3::{
    prelude::*,
    types::{IntoPyDict, PyDict},
};
use serde_json::Value;

static EMBEDDINGS: Lazy<Mutex<HashMap<String, Embedding>>> = Lazy::new(Default::default);

#[derive(Clone)]
pub struct Embedding {
    ptr: PyObject,
    kwargs: Value,
    backend: Backend,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Backend {
    Instructor(String),
    SentenceTransformers,
    Transformers,
}

impl Embedding {
    pub fn new(transformer: &str, kwargs: &Value) -> Result<Self> {
        if !kwargs.is_object() {
            bail!("`expect `kwargs` to be JSON object");
        }

        let mut kwargs = kwargs.clone();
        let backend = Backend::new(transformer, &mut kwargs)?;

        // Acquire the GIL to create the model
        let ptr = Python::with_gil(|py| match backend {
            Backend::Instructor(_) => Self::create_instructor(transformer, py),
            Backend::SentenceTransformers => Self::create_sentence_transformer(transformer, py),
            Backend::Transformers => Self::create_automodel(transformer, &mut kwargs, py),
        })?;

        Ok(Self {
            ptr,
            kwargs,
            backend,
        })
    }

    /// Equivalent to:
    /// ```python
    /// from InstructorEmbedding import INSTRUCTOR
    /// INSTRUCTOR(transformer)
    /// ```
    fn create_instructor(transformer: &str, py: Python) -> PyResult<PyObject> {
        PyModule::import(py, "InstructorEmbedding")?
            .getattr("INSTRUCTOR")?
            .call1((transformer,))?
            .extract()
    }

    /// Equivalent to:
    /// ```python
    /// from sentence_transformers import SentenceTransformers
    /// SentenceTransformers(transformer)
    /// ```
    fn create_sentence_transformer(transformer: &str, py: Python) -> PyResult<PyObject> {
        PyModule::import(py, "sentence_transformers")?
            .getattr("SentenceTransformer")?
            .call1((transformer,))?
            .extract()
    }

    /// Equivalent to:
    /// ```python
    /// from transformers import AutoModel
    /// AutoModel.from_pretrained(transformer, trust_remote_code=trust_remote_code)
    /// ```
    fn create_automodel(transformer: &str, kwargs: &mut Value, py: Python) -> PyResult<PyObject> {
        // take `trust_remote_code` out and use it, if present
        let trust_remote_code = kwargs
            .as_object_mut()
            .is_some_and(|v| matches!(v.remove("trust_remote_code"), Some(Value::Bool(true))));

        let kwargs = [("trust_remote_code", trust_remote_code)].into_py_dict(py);

        PyModule::import(py, "transformers")?
            .getattr("AutoModel")?
            .getattr("from_pretrained")?
            .call((transformer,), Some(kwargs))?
            .extract()
    }

    /// Equivalent to:
    /// ```python
    /// model.encode(inputs, **kwargs)
    /// ````
    pub fn encode(&self, inputs: &[&str]) -> Result<Vec<Vec<f32>>> {
        // Acquire the GIL to use the model
        Python::with_gil(|py| {
            let kwargs = value_to_pydict(&self.kwargs, py).context("converting Value to PyDict")?;
            let inputs: PyObject = match &self.backend {
                Backend::Instructor(instruction) => inputs
                    .iter()
                    .map(|i| vec![instruction.clone(), i.to_string()])
                    .collect::<Vec<_>>()
                    .into_py(py),
                _ => inputs
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .into_py(py),
            };

            Ok(self
                .ptr
                .getattr(py, "encode")?
                .call(py, (inputs,), Some(kwargs))?
                .extract(py)?)
        })
    }
}

impl Backend {
    pub fn new(transformer: &str, kwargs: &mut Value) -> Result<Self> {
        Ok(if transformer.starts_with("hkunlp/instructor") {
            // Unwrap is OK because constructor ensures kwargs is an object
            let instruction = kwargs
                .as_object_mut()
                .unwrap()
                .remove("INSTRUCTION")
                .and_then(|v| match v {
                    Value::String(s) => Some(s.to_string()),
                    _ => None,
                })
                .ok_or_else(|| {
                    anyhow!("Instructor model requires `INSTRUCTION` string in `kwargs`")
                })?;

            Self::Instructor(instruction)
        } else if kwargs.as_object_mut().is_some_and(
            |obj| matches!(obj.remove("backend"), Some(Value::String(s)) if s == "transformers"),
        ) {
            Self::Transformers
        } else {
            Self::SentenceTransformers
        })
    }
}

pub fn embed(transformer: &str, inputs: Vec<&str>, kwargs: &Value) -> Result<Vec<Vec<f32>>> {
    crate::bindings::python::activate()?;

    let embeddings = EMBEDDINGS.lock();
    let embedding = if !embeddings.contains_key(transformer) {
        drop(embeddings); // Release the lock while we load/download the embedding model
        let embedding = Embedding::new(transformer, kwargs)?;
        let mut embeddings = EMBEDDINGS.lock();
        embeddings.insert(transformer.to_string(), embedding.clone());
        embedding
    } else {
        // Unwrap is OK because we checked it existed.
        embeddings.get(transformer).unwrap().clone()
    };

    embedding.encode(&inputs)
}

fn value_to_pydict(v: &Value, py: Python<'py>) -> Result<&'py PyDict> {
    let d = PyDict::new(py);
    for (k, v) in v
        .as_object()
        .ok_or_else(|| anyhow!("`kwargs` expected as JSON object"))?
    {
        match v {
            Value::Bool(b) => d.set_item(k, b)?,
            Value::Number(n) => {
                if n.is_i64() {
                    d.set_item(k, n.as_i64().unwrap())?
                } else if n.is_u64() {
                    d.set_item(k, n.as_u64().unwrap())?
                } else {
                    d.set_item(k, n.as_f64().unwrap())?
                }
            }
            Value::String(s) => d.set_item(k, s)?,
            _ => bail!("`kwargs` contains an unsupported datatype"),
        };
    }
    Ok(d.extract()?)
}

#[cfg(test)]
mod tests {
    use std::{path::Path, time::Instant};

    use super::*;

    use serde_json::json;

    fn activate_venv() {
        if let Some(venv) = option_env!("PGML_VENV") {
            Python::with_gil(|py| {
                let locals = PyDict::new(py);
                let activate_this_file = Path::new(venv).join("bin/activate_this.py");
                locals
                    .set_item("activate_this_file", activate_this_file)
                    .unwrap();
                py.eval(
                    "exec(open(activate_this_file).read(), {'__file__': activate_this_file})",
                    None,
                    Some(locals),
                )
                .unwrap();
            })
        } else {
            println!("if Python import errors occur, set PGML_VENV to your virtualenv");
        }
    }

    #[test]
    fn test_backend() {
        assert!(matches!(
            Backend::new(
                "hkunlp/instructor-xl",
                &mut json!({"INSTRUCTION": "Represent the Science statement: "})
            )
            .unwrap(),
            Backend::Instructor(_)
        ));
        assert_eq!(
            Backend::new("intfloat/e5-small", &mut json!({})).unwrap(),
            Backend::SentenceTransformers
        );
        assert_eq!(
            Backend::new(
                "jinaai/jina-embeddings-v2-base-en",
                &mut json!({"backend": "transformers"})
            )
            .unwrap(),
            Backend::Transformers
        );
    }

    #[test]
    fn test_jina() {
        activate_venv();
        let model = "jinaai/jina-embeddings-v2-base-en";
        let kwargs = json!({"trust_remote_code": true, "backend": "transformers"});
        let embedding = Embedding::new(model, &kwargs).unwrap();
        let inputs = vec!["PostgresML makes machine learning easy and fun!"];
        embedding.encode(&inputs).unwrap();
    }

    #[test]
    fn test_intfloat() {
        activate_venv();
        let model = "intfloat/e5-small";
        let kwargs = json!({});
        let embedding = Embedding::new(model, &kwargs).unwrap();
        let inputs = vec!["PostgresML makes machine learning easy and fun!"];
        embedding.encode(&inputs).unwrap();
    }

    #[test]
    fn test_instructor() {
        activate_venv();
        let model = "hkunlp/instructor-base";
        let kwargs = json!({"INSTRUCTION": "Represent the Science sentence: "});
        let embedding = Embedding::new(model, &kwargs).unwrap();
        let inputs = vec!["PostgresML makes machine learning easy and fun!"];
        embedding.encode(&inputs).unwrap();
    }
}
