use anyhow::Result;
use pyo3::prelude::*;
use pyo3::ffi::c_str;
use pyo3::types::PyString;

use crate::create_pymodule;

create_pymodule!("/src/bindings/langchain/langchain.py");

pub fn chunk(splitter: &str, text: &str, kwargs: &serde_json::Value) -> Result<Vec<String>> {

    Python::with_gil(|py| -> Result<Vec<String>> {
        let chunk: Py<PyAny> = get_module!(PY_MODULE).getattr(py, "chunk")?;
        let splitter = PyString::new(py, splitter);
        let text = PyString::new(py, text);
        let kwargs = PyString::new(py, serde_json::to_string(kwargs)?.as_str());

        Ok(chunk
            .call1(py,(splitter, text, kwargs))?
            .extract(py)?)
    })
}
