//! Use virtualenv.

use once_cell::sync::Lazy;
use pgrx::*;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::config::get_config;

static CONFIG_NAME: &'static str = "pgml.venv";

static PY_MODULE: Lazy<Py<PyModule>> = Lazy::new(|| {
    Python::with_gil(|py| -> Py<PyModule> {
        let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bindings/venv.py"));

        PyModule::from_code(py, src, "", "").unwrap().into()
    })
});

pub fn activate_venv(venv: &str) -> bool {
    Python::with_gil(|py| -> bool {
        let activate_venv: Py<PyAny> = PY_MODULE.getattr(py, "activate_venv").unwrap().into();
        let result: Py<PyAny> = activate_venv
            .call1(py, PyTuple::new(py, &[venv.to_string().into_py(py)]))
            .unwrap();

        result.extract(py).unwrap()
    })
}

pub fn activate() -> bool {
    match get_config(CONFIG_NAME) {
        Some(venv) => activate_venv(&venv),
        None => false,
    }
}
