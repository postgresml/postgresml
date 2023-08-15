//! Use virtualenv.

use anyhow::Result;
use once_cell::sync::Lazy;
use pgrx::*;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::config::get_config;
use crate::{bindings::TracebackError, create_pymodule};

static CONFIG_NAME: &str = "pgml.venv";

create_pymodule!("/src/bindings/venv.py");

pub fn activate_venv(venv: &str) -> Result<bool> {
    Python::with_gil(|py| {
        let activate_venv: Py<PyAny> = get_module!(PY_MODULE).getattr(py, "activate_venv")?;
        let result: Py<PyAny> =
            activate_venv.call1(py, PyTuple::new(py, &[venv.to_string().into_py(py)]))?;

        Ok(result.extract(py)?)
    })
}

pub fn activate() -> Result<bool> {
    match get_config(CONFIG_NAME) {
        Some(venv) => activate_venv(&venv),
        None => Ok(false),
    }
}

pub fn freeze() -> Result<Vec<String>> {
    Python::with_gil(|py| {
        let freeze = get_module!(PY_MODULE).getattr(py, "freeze")?;
        let result = freeze.call0(py)?;

        Ok(result.extract(py)?)
    })
}
