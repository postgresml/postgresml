use super::whitelist;
use super::TracebackError;
use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
create_pymodule!("/src/bindings/transformers/transformers.py");

pub fn transform(
    task: &serde_json::Value,
    args: &serde_json::Value,
    inputs: Vec<&str>,
) -> Result<serde_json::Value> {
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

pub fn transform_stream(
    task: &serde_json::Value,
    args: &serde_json::Value,
    input: &str,
) -> Result<Py<PyAny>> {
    crate::bindings::python::activate()?;

    whitelist::verify_task(task)?;

    let task = serde_json::to_string(task)?;
    let args = serde_json::to_string(args)?;
    let inputs = serde_json::to_string(&vec![input])?;

    Python::with_gil(|py| -> Result<Py<PyAny>> {
        let transform: Py<PyAny> = get_module!(PY_MODULE)
            .getattr(py, "transform")
            .format_traceback(py)?;

        let output = transform
            .call1(
                py,
                PyTuple::new(
                    py,
                    &[
                        task.into_py(py),
                        args.into_py(py),
                        inputs.into_py(py),
                        true.into_py(py),
                    ],
                ),
            )
            .format_traceback(py)?;

        Ok(output)
    })
}
