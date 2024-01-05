use super::whitelist;
use super::TracebackError;
use anyhow::Result;
use pgrx::*;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict, PyTuple};

create_pymodule!("/src/bindings/transformers/transformers.py");

pub struct TransformStreamIterator {
    locals: Py<PyDict>,
}

impl TransformStreamIterator {
    pub fn new(python_iter: Py<PyAny>) -> Self {
        let locals = Python::with_gil(|py| -> Result<Py<PyDict>, PyErr> {
            Ok([("python_iter", python_iter)].into_py_dict(py).into())
        })
        .map_err(|e| error!("{e}"))
        .unwrap();
        Self { locals }
    }
}

impl Iterator for TransformStreamIterator {
    type Item = JsonB;
    fn next(&mut self) -> Option<Self::Item> {
        // We can unwrap this becuase if there is an error the current transaction is aborted in the map_err call
        Python::with_gil(|py| -> Result<Option<JsonB>, PyErr> {
            let code = "next(python_iter)";
            let res: &PyAny = py.eval(code, Some(self.locals.as_ref(py)), None)?;
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
    whitelist::verify_task(task)?;

    let task = serde_json::to_string(task)?;
    let args = serde_json::to_string(args)?;
    let inputs = serde_json::to_string(&inputs)?;

    let results = Python::with_gil(|py| -> Result<String> {
        let transform: Py<PyAny> = get_module!(PY_MODULE).getattr(py, "transform").format_traceback(py)?;

        let output = transform
            .call1(
                py,
                PyTuple::new(py, &[task.into_py(py), args.into_py(py), inputs.into_py(py)]),
            )
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

    let task = serde_json::to_string(task)?;
    let args = serde_json::to_string(args)?;
    let input = serde_json::to_string(&input)?;

    Python::with_gil(|py| -> Result<Py<PyAny>> {
        let transform: Py<PyAny> = get_module!(PY_MODULE).getattr(py, "transform").format_traceback(py)?;

        let output = transform
            .call1(
                py,
                PyTuple::new(
                    py,
                    &[task.into_py(py), args.into_py(py), input.into_py(py), true.into_py(py)],
                ),
            )
            .format_traceback(py)?;

        Ok(output)
    })
}

pub fn transform_stream_iterator<T: serde::Serialize>(
    task: &serde_json::Value,
    args: &serde_json::Value,
    input: T,
) -> Result<TransformStreamIterator> {
    let python_iter = transform_stream(task, args, input).map_err(|e| error!("{e}")).unwrap();
    Ok(TransformStreamIterator::new(python_iter))
}
