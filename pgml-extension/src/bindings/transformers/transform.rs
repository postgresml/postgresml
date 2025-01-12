use super::whitelist;
use super::TracebackError;
use anyhow::Result;
use pgrx::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};
use pyo3::ffi::c_str;

create_pymodule!("/src/bindings/transformers/transformers.py");

pub struct TransformStreamIterator {
    locals: Py<PyDict>,  // Store owned version instead of Bound
}

impl TransformStreamIterator {
    pub fn new(python_iter: Py<PyAny>) -> Self {
        let locals = Python::with_gil(|py| {
            let dict = PyDict::new(py);
            dict.set_item("python_iter", &python_iter)?;
            Ok::<Py<PyDict>, PyErr>(dict.into())
        })
            .map_err(|e: PyErr| error!("{e}"))
            .unwrap();

        Self { locals }
    }
}

impl Iterator for TransformStreamIterator {
    type Item = JsonB;

    fn next(&mut self) -> Option<Self::Item> {
        Python::with_gil(|py| -> Result<Option<JsonB>, PyErr> {
            let locals = self.locals.bind(py);  // Get Bound reference when needed
            let code = c_str!("next(python_iter)");
            let res = py.eval(code, Some(&locals), None)?;
            if res.is_none() {
                Ok(None)
            } else {
                let res: Vec<String> = res.extract()?;
                Ok(Some(JsonB(serde_json::to_value(res).unwrap())))
            }
        })
            .map_err(|e| error!("{e}"))
            .unwrap()
    }
}

pub fn transform<T: serde::Serialize>(
    task: &serde_json::Value,
    args: &serde_json::Value,
    inputs: T,
) -> Result<serde_json::Value> {
    let results = Python::with_gil(|py| -> Result<String> {
        let transform = get_module!(PY_MODULE).getattr(py, "transform").format_traceback(py)?;
        let task = PyString::new(py, &serde_json::to_string(task)?);
        let args = PyString::new(py, &serde_json::to_string(args)?);
        let inputs = PyString::new(py, &serde_json::to_string(&inputs)?);

        let output = transform
            .call1(py, (task, args, inputs))
            .format_traceback(py)?;

        output.extract(py).format_traceback(py)
    })?;

    Ok(serde_json::from_str(&results)?)
}

pub fn transform_stream<T: serde::Serialize>(
    task: &serde_json::Value,
    args: &serde_json::Value,
    input: T,
) -> Result<Py<PyAny>> {
    whitelist::verify_task(task)?;

    Python::with_gil(|py| -> Result<Py<PyAny>> {
        let transform: Py<PyAny> = get_module!(PY_MODULE).getattr(py, "transform").format_traceback(py)?;
        let task = PyString::new(py, &serde_json::to_string(task)?);
        let args = PyString::new(py, &serde_json::to_string(args)?);
        let input = PyString::new(py, &serde_json::to_string(&input)?);

        let output = transform
            .call1(py, (task, args, input, true))
            .format_traceback(py)?;

        Ok(output)
    })
}

pub fn transform_stream_iterator<'a, T: serde::Serialize>(
    task: &'a serde_json::Value,
    args: &'a serde_json::Value,
    input: T,
) -> Result<TransformStreamIterator> {
    let python_iter = transform_stream(task, args, input).map_err(|e| error!("{e}")).unwrap();
    Ok(TransformStreamIterator::new(python_iter))
}
