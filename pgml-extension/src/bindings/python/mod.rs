//! Use virtualenv.

use anyhow::Result;
use pgrx::iter::TableIterator;
use pgrx::*;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::config::PGML_VENV;
use crate::create_pymodule;

create_pymodule!("/src/bindings/python/python.py");

pub fn activate_venv(venv: &str) -> Result<bool> {
    Python::with_gil(|py| {
        let activate_venv: Py<PyAny> = get_module!(PY_MODULE).getattr(py, "activate_venv")?;
        let result: Py<PyAny> = activate_venv.call1(py, PyTuple::new(py, &[venv.to_string().into_py(py)]))?;

        Ok(result.extract(py)?)
    })
}

pub fn activate() -> Result<bool> {
    match PGML_VENV.1.get() {
        Some(venv) => activate_venv(&venv.to_string_lossy()),
        None => Ok(false),
    }
}

pub fn pip_freeze() -> Result<TableIterator<'static, (name!(package, String),)>> {
    let packages = Python::with_gil(|py| -> Result<Vec<String>> {
        let freeze = get_module!(PY_MODULE).getattr(py, "freeze")?;
        let result = freeze.call0(py)?;

        Ok(result.extract(py)?)
    })?;

    Ok(TableIterator::new(packages.into_iter().map(|package| (package,))))
}

pub fn validate_dependencies() -> Result<bool> {
    Python::with_gil(|py| {
        let sys = PyModule::import(py, "sys").unwrap();
        let version: String = sys.getattr("version").unwrap().extract().unwrap();
        info!("Python version: {version}");
        for module in ["xgboost", "lightgbm", "numpy", "sklearn"] {
            match py.import(module) {
                Ok(_) => (),
                Err(e) => {
                    panic!("The {module} package is missing. Install it with `sudo pip3 install {module}`\n{e}");
                }
            }
        }
    });

    let sklearn = package_version("sklearn")?;
    let xgboost = package_version("xgboost")?;
    let lightgbm = package_version("lightgbm")?;
    let numpy = package_version("numpy")?;

    info!("Scikit-learn {sklearn}, XGBoost {xgboost}, LightGBM {lightgbm}, NumPy {numpy}",);

    Ok(true)
}

pub fn version() -> Result<String> {
    Python::with_gil(|py| {
        let sys = PyModule::import(py, "sys").unwrap();
        let version: String = sys.getattr("version").unwrap().extract().unwrap();
        Ok(version)
    })
}

pub fn package_version(name: &str) -> Result<String> {
    Python::with_gil(|py| {
        let package = py.import(name)?;
        Ok(package.getattr("__version__")?.extract()?)
    })
}
