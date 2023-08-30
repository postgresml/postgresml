//! Use virtualenv.

use anyhow::Result;
use once_cell::sync::Lazy;
use pgrx::*;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::config::get_config;
use crate::{bindings::TracebackError, create_pymodule};

static CONFIG_NAME: &str = "pgml.venv";

create_pymodule!("/src/bindings/python/python.py");

pub fn activate_venv(venv: &str) -> Result<bool> {
    Python::with_gil(|py| {
        let activate_venv: Py<PyAny> = get_module!(PY_MODULE).getattr(py, "activate_venv")?;
        let result: Py<PyAny> =
            activate_venv.call1(py, PyTuple::new(py, &[venv.to_string().into_py(py)]))?;

        Ok(result.extract(py)?)
    })
}

pub fn activate() -> Result<Option<String>> {
    match get_config(CONFIG_NAME) {
        Some(venv) => match activate_venv(&venv) {
            Ok(_) => Ok(Some(venv)),
            Err(_) => Ok(None),
        },
        None => Ok(None),
    }
}

pub fn pip_freeze() -> Result<Vec<String>> {
    activate()?;
    Ok(Python::with_gil(|py| -> Result<Vec<String>> {
        let freeze = get_module!(PY_MODULE).getattr(py, "freeze")?;
        let result = freeze.call0(py)?;

        Ok(result.extract(py)?)
    })?)
}

pub fn validate_dependencies() -> Result<bool> {
    let venv = activate()?;

    if let Some(venv) = venv {
        info!("Using virtual environment located in: {}", venv);
    } else {
        info!("Using system Python environment");
    }

    let validated = Python::with_gil(|py| {
        let sys = PyModule::import(py, "sys").unwrap();
        let version: String = sys.getattr("version").unwrap().extract().unwrap();
        info!("Python version: {version}");
        for module in [
            "xgboost",
            "lightgbm",
            "numpy",
            "sklearn",
            "transformers",
            "datasets",
            "torch",
            "sentence_transformers",
            "InstructorEmbedding",
        ] {
            match py.import(module) {
                Ok(_) => (),
                Err(e) => {
                    info!(
                        "The {module} package is missing. Install it with `sudo pip3 install {module}`: {e}"
                    );
                    info!("Installing without Python dependencies, some functionality may be unavailable");
                    return false;
                }
            }
        }

        true
    });

    if validated {
        let sklearn = package_version("sklearn")?;
        let xgboost = package_version("xgboost")?;
        let lightgbm = package_version("lightgbm")?;
        let numpy = package_version("numpy")?;
        let transformers = package_version("transformers")?;
        let torch = package_version("torch")?;
        let sentence_transformers = package_version("sentence_transformers")?;

        info!("PyTorch {torch}, Transformers {transformers}, Sentence Transformers {sentence_transformers}, Scikit-learn {sklearn}, XGBoost {xgboost}, LightGBM {lightgbm}, NumPy {numpy}",);
    }

    Ok(true)
}

pub fn version() -> Result<String> {
    activate()?;
    Python::with_gil(|py| {
        let sys = PyModule::import(py, "sys").unwrap();
        let version: String = sys.getattr("version").unwrap().extract().unwrap();
        Ok(version)
    })
}

pub fn package_version(name: &str) -> Result<String> {
    activate()?;
    Python::with_gil(|py| {
        let package = py.import(name)?;
        Ok(package.getattr("__version__")?.extract()?)
    })
}

pub fn cuda_available() -> Result<bool> {
    activate()?;
    Python::with_gil(|py| {
        let cuda_available: Py<PyAny> = get_module!(PY_MODULE).getattr(py, "cuda_available")?;
        let result: Py<PyAny> = cuda_available.call0(py)?;

        Ok(result.extract(py)?)
    })
}
