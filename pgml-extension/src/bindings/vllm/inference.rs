use parking_lot::Mutex;
use pyo3::prelude::*;
use serde_json::{json, Value};

use super::LLM;

/// Cache a single model per client process. vLLM does not allow multiple, simultaneous models to be loaded.
/// See GH issue, https://github.com/vllm-project/vllm/issues/565
static MODEL: Mutex<Option<LLM>> = Mutex::new(None);

pub fn vllm_inference(task: &Value, inputs: &[&str]) -> PyResult<Value> {
    crate::bindings::python::activate().expect("python venv activate");
    let mut model = MODEL.lock();

    let llm = match get_model_name(&model, task) {
        ModelName::Same => model.as_mut().expect("ModelName::Same as_mut"),
        ModelName::Different(name) => {
            if let Some(llm) = model.take() {
                // delete old model, exists
                destroy_model_parallel(llm)?;
            }
            // make new model
            let llm = LLM::new(&name)?;
            model.insert(llm)
        }
    };

    let outputs = llm
        .generate(&inputs, None)?
        .iter()
        .map(|o| {
            o.outputs()
                .expect("RequestOutput::outputs()")
                .iter()
                .map(|o| o.text().expect("CompletionOutput::text()"))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<Vec<_>>>();

    Ok(json!(outputs))
}

/// Determine if the "model" specified in the task is the same model as the one cached.
/// 
/// # Panic
/// This function panics if:
/// - `task` is not an object
/// - "model" key is missing from `task` object
/// - "model" value is not a str
fn get_model_name<M>(model: &M, task: &Value) -> ModelName
where
    M: std::ops::Deref<Target = Option<LLM>>,
{
    let name = task.as_object()
        .expect("`task` is an object")
        .get("model")
        .expect("model key is present")
        .as_str()
        .expect("model value is a str");

    if matches!(model.as_ref(), Some(llm) if llm.model() == name) {
        ModelName::Same
    } else {
        ModelName::Different(name.to_string())
    }
}

enum ModelName {
    Same,
    Different(String),
}

// See https://github.com/vllm-project/vllm/issues/565#issuecomment-1725174811
fn destroy_model_parallel(llm: LLM) -> PyResult<()> {
    Python::with_gil(|py| {
        PyModule::import(py, "vllm")?
            .getattr("model_executor")?
            .getattr("parallel_utils")?
            .getattr("parallel_state")?
            .getattr("destroy_model_parallel")?
            .call0()?;
        drop(llm);
        PyModule::import(py, "gc")?.getattr("collect")?.call0()?;
        Ok(())
    })
}
