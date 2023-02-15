use pyo3::prelude::*;
use pyo3::types::PyTuple;

use pgx::iter::{SetOfIterator, TableIterator};
use pgx::*;

pub fn transform(
    task: &serde_json::Value,
    args: &serde_json::Value,
    inputs: &Vec<String>,
) -> serde_json::Value {
    let module = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/bindings/transformers.py"
    ));

    let task = serde_json::to_string(task).unwrap();
    let args = serde_json::to_string(args).unwrap();
    let inputs = serde_json::to_string(inputs).unwrap();

    let results = Python::with_gil(|py| -> String {
        let module = PyModule::from_code(py, module, "", "").unwrap();
        let transformer: Py<PyAny> = module.getattr("transform").unwrap().into();

        transformer
            .call1(
                py,
                PyTuple::new(
                    py,
                    &[task.into_py(py), args.into_py(py), inputs.into_py(py)],
                ),
            )
            .unwrap()
            .extract(py)
            .unwrap()
    });
    serde_json::from_str(&results).unwrap()
}

pub fn load_dataset() {

}
