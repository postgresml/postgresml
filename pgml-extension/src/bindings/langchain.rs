use once_cell::sync::Lazy;
use pgrx::*;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

static PY_MODULE: Lazy<Py<PyModule>> = Lazy::new(|| {
    Python::with_gil(|py| -> Py<PyModule> {
        let src = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/bindings/langchain.py"
        ));

        PyModule::from_code(py, src, "", "").unwrap().into()
    })
});

pub fn chunk(splitter: &str, text: &str, kwargs: &serde_json::Value) -> Vec<String> {
    crate::bindings::venv::activate();

    let kwargs = serde_json::to_string(kwargs).unwrap();

    Python::with_gil(|py| -> Vec<String> {
        let chunk: Py<PyAny> = PY_MODULE.getattr(py, "chunk").unwrap().into();

        chunk
            .call1(
                py,
                PyTuple::new(
                    py,
                    &[splitter.into_py(py), text.into_py(py), kwargs.into_py(py)],
                ),
            )
            .unwrap()
            .extract(py)
            .unwrap()
    })
}
