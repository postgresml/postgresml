use pyo3::types::{PyDict, PyFloat, PyInt, PyList, PyString};
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
        match &self.0 {
            serde_json::Value::Bool(x) => x.to_object(py),
            serde_json::Value::Number(x) => x.as_f64().to_object(py),
            serde_json::Value::String(x) => x.to_object(py),
            serde_json::Value::Array(x) => {
                let list = PyList::empty(py);
                for v in x.iter() {
                    list.append(Json(v.clone()).to_object(py)).unwrap();
                }
                list.to_object(py)
            },
            serde_json::Value::Object(x) => {
                let dict = PyDict::new(py);
                for (k, v) in x.iter() {
                    dict.set_item(k, Json(v.clone()).to_object(py)).unwrap();
                }
                dict.to_object(py)
            },
            _ => panic!("Unsupported type for JSON conversion")
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// PY to Rust //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////
impl FromPyObject<'_> for Json {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        if ob.is_instance_of::<PyDict>()? {
            let dict: &PyDict = ob.downcast()?;
            let mut json = serde_json::Map::new();
            for (key, value) in dict.iter() {
                let value = Json::extract(value)?;
                json.insert(String::extract(key)?, value.0);
            }
            Ok(Self(serde_json::Value::Object(json)))
        } else if ob.is_instance_of::<PyBool>()? {
            let value = bool::extract(ob)?;
            Ok(Self(serde_json::Value::Bool(value)))
        } else if ob.is_instance_of::<PyInt>()? {
            let value = i64::extract(ob)?;
            Ok(Self(serde_json::Value::Number(value.into())))
        } else if ob.is_instance_of::<PyFloat>()? {
            let value = f64::extract(ob)?;
            let value = serde_json::value::Number::from_f64(value)
                .expect("Could not convert f64 to serde_json::Number");
            Ok(Self(serde_json::Value::Number(value)))
        } else if ob.is_instance_of::<PyString>()? {
            let value = String::extract(ob)?;
            Ok(Self(serde_json::Value::String(value)))
        } else if ob.is_instance_of::<PyList>()? {
            let value = ob.downcast::<PyList>()?;
            let mut json_values = Vec::new();
            for v in value {
                let v = v.extract::<Json>()?;
                json_values.push(v.0);
            }
            Ok(Self(serde_json::Value::Array(json_values)))
        } else {
            panic!("Unsupported type for JSON conversion");
        }
    }
}
