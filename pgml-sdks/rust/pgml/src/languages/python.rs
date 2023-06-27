use pyo3::types::{PyDict, PyFloat, PyInt, PyString};
use pyo3::{prelude::*, types::PyBool};

use crate::types::{DateTime, Json};

////////////////////////////////////////////////////////////////////////////////
// Rust to PY //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////
impl ToPyObject for DateTime {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.timestamp().to_object(py)
    }
}

impl ToPyObject for Json {
    fn to_object(&self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        for (k, v) in self
            .0
            .as_object()
            .expect("We currently only support json objects")
            .iter()
        {
            match v {
                // TODO: Support more types like nested objects
                serde_json::Value::Number(x) => dict.set_item(k, x.as_i64().unwrap()).unwrap(),
                serde_json::Value::Bool(x) => dict.set_item(k, x).unwrap(),
                serde_json::Value::String(x) => dict.set_item(k, x).unwrap(),
                _ => {}
            }
        }
        dict.to_object(py)
    }
}

////////////////////////////////////////////////////////////////////////////////
// PY to Rust //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////
impl FromPyObject<'_> for Json {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let dict: &PyDict = ob.downcast()?;
        let mut json = serde_json::Map::new();
        for (key, value) in dict.iter() {
            // TODO: Support more types
            if value.is_instance_of::<PyBool>()? {
                let value = bool::extract(value)?;
                json.insert(String::extract(key)?, serde_json::Value::Bool(value));
            } else if value.is_instance_of::<PyInt>()? {
                let value = i64::extract(value)?;
                json.insert(
                    String::extract(key)?,
                    serde_json::Value::Number(value.into()),
                );
            } else if value.is_instance_of::<PyFloat>()? {
                let value = f64::extract(value)?;
                let value = serde_json::value::Number::from_f64(value)
                    .expect("Could not convert f64 to serde_json::Number");
                json.insert(String::extract(key)?, serde_json::Value::Number(value));
            } else if value.is_instance_of::<PyString>()? {
                let value = String::extract(value)?;
                json.insert(String::extract(key)?, serde_json::Value::String(value));
            } else {
                panic!("Unsupported type for json conversion");
            }
        }
        Ok(Self(serde_json::Value::Object(json)))
    }
}
