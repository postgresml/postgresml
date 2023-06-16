use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::models::{DateTime, JsonHashMap};

////////////////////////////////////////////////////////////////////////////////
// Rust to PY //////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////

impl ToPyObject for DateTime {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.timestamp().to_object(py)
    }
}

impl ToPyObject for JsonHashMap {
    fn to_object(&self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        for (k, v) in self.0.iter() {
            dict.set_item(k, v).unwrap();
        }
        dict.to_object(py)
    }
}
