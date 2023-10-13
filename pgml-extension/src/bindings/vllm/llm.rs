use pyo3::{prelude::*, types::PyDict};
use serde_json::Value;

use super::SamplingParams;

pub struct LLMBuilder {
    model: String,
    tokenizer: Option<String>,
    tokenizer_mode: TokenizerMode,
    trust_remote_code: bool,
    tensor_parallel_size: u8,
    dtype: String,
    quantization: Option<Quantization>,
    revision: Option<String>,
    seed: u64,
    gpu_memory_utilization: f64,
    swap_space: u32,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TokenizerMode {
    Auto,
    Slow,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Quantization {
    Awq,
}

pub struct LLM {
    inner: PyObject,
}

impl LLMBuilder {
    /// Create a builder for a model with the name or path of a HuggingFace
    /// Transformers model.
    pub fn new(model: &str) -> Self {
        Self {
            model: model.to_string(),
            tokenizer: None,
            tokenizer_mode: TokenizerMode::Auto,
            trust_remote_code: false,
            tensor_parallel_size: 1,
            dtype: "auto".to_string(),
            quantization: None,
            revision: None,
            seed: 0,
            gpu_memory_utilization: 0.9,
            swap_space: 4,
        }
    }

    /// The name or path of a HuggingFace Transformers tokenizer.
    pub fn tokenizer(mut self, tokenizer: &str) -> Self {
        self.tokenizer = Some(tokenizer.to_string());
        self
    }

    /// The tokenizer mode. "auto" will use the fast tokenizer if available, and
    /// "slow" will always use the slow tokenizer.
    pub fn tokenizer_mode(mut self, tokenizer_mode: TokenizerMode) -> Self {
        self.tokenizer_mode = tokenizer_mode;
        self
    }

    /// Trust remote code (e.g., from HuggingFace) when downloading the model
    /// and tokenizer.
    pub fn trust_remote_code(mut self, trust_remote_code: bool) -> Self {
        self.trust_remote_code = trust_remote_code;
        self
    }

    /// The number of GPUs to use for distributed execution with tensor
    /// parallelism.
    pub fn tensor_parallel_size(mut self, tensor_parallel_size: u8) -> Self {
        self.tensor_parallel_size = tensor_parallel_size;
        self
    }

    /// The data type for the model weights and activations. Currently,
    /// we support `float32`, `float16`, and `bfloat16`. If `auto`, we use
    /// the `torch_dtype` attribute specified in the model config file.
    /// However, if the `torch_dtype` in the config is `float32`, we will
    /// use `float16` instead.
    pub fn dtype(mut self, dtype: &str) -> Self {
        self.dtype = dtype.to_string();
        self
    }

    /// The method used to quantize the model weights. Currently,
    /// we support "awq". If None, we assume the model weights are not
    /// quantized and use `dtype` to determine the data type of the weights.
    pub fn quantization(mut self, quantization: Quantization) -> Self {
        self.quantization = Some(quantization);
        self
    }

    /// The specific model version to use. It can be a branch name,
    /// a tag name, or a commit id.
    pub fn revision(mut self, revision: &str) -> Self {
        self.revision = Some(revision.to_string());
        self
    }

    /// The seed to initialize the random number generator for sampling.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// The ratio (between 0 and 1) of GPU memory to
    /// reserve for the model weights, activations, and KV cache. Higher
    /// values will increase the KV cache size and thus improve the model's
    /// throughput. However, if the value is too high, it may cause out-of-
    /// memory (OOM) errors.
    pub fn gpu_memory_utilization(mut self, gpu_memory_utilization: f64) -> Self {
        self.gpu_memory_utilization = gpu_memory_utilization;
        self
    }

    /// The size (GiB) of CPU memory per GPU to use as swap space.
    /// This can be used for temporarily storing the states of the requests
    /// when their `best_of` sampling parameters are larger than 1. If all
    /// requests will have `best_of=1`, you can safely set this to 0.
    /// Otherwise, too small values may cause out-of-memory (OOM) errors.
    pub fn swap_space(mut self, swap_space: u32) -> Self {
        self.swap_space = swap_space;
        self
    }

    /// Create a [`LLM`] from the [`LLMBuilder`]
    pub fn build(self) -> PyResult<LLM> {
        let inner = Python::with_gil(|py| -> PyResult<PyObject> {
            let kwargs = PyDict::new(py);
            kwargs.set_item("model", self.model)?;
            kwargs.set_item("tokenizer", self.tokenizer)?;
            kwargs.set_item("tokenizer_mode", self.tokenizer_mode)?;
            kwargs.set_item("trust_remote_code", self.trust_remote_code)?;
            kwargs.set_item("tensor_parallel_size", self.tensor_parallel_size)?;
            kwargs.set_item("dtype", self.dtype)?;
            kwargs.set_item("quantization", self.quantization)?;
            kwargs.set_item("revision", self.revision)?;
            kwargs.set_item("seed", self.seed)?;
            kwargs.set_item("gpu_memory_utilization", self.gpu_memory_utilization)?;
            kwargs.set_item("swap_space", self.swap_space)?;

            let vllm = PyModule::import(py, "vllm")?;
            vllm.getattr("LLM")?.call((), Some(kwargs))?.extract()
        })?;

        Ok(LLM { inner })
    }
}

impl LLM {
    /// Create an LLM for a model with the name or path of a HuggingFace
    /// Transformers model.
    pub fn new(model: &str) -> PyResult<Self> {
        LLMBuilder::new(model).build()
    }

    /// Generates the completions for the input prompts.
    ///
    /// ### NOTE
    /// This automatically batches the given prompts, considering the memory
    /// constraint. For the best performance, put all of your prompts into a
    /// single list and pass it to this method.
    pub fn generate(
        &self,
        prompts: &[&str],
        params: Option<&SamplingParams>,
    ) -> PyResult<Vec<String>> {
        let prompts: Vec<_> = prompts.iter().map(|s| s.to_string()).collect();

        Python::with_gil(|py| {
            let kwargs = PyDict::new(py);
            kwargs.set_item("prompts", prompts)?;
            kwargs.set_item("sampling_params", params)?;

            let outputs: Vec<PyObject> = self
                .inner
                .getattr(py, "generate")?
                .call(py, (), Some(kwargs))?
                .extract(py)?;

            outputs
                .iter()
                .map(|output| -> PyResult<String> {
                    let outputs: Vec<PyObject> = output.getattr(py, "outputs")?.extract(py)?;
                    outputs.first().unwrap().getattr(py, "text")?.extract(py)
                })
                .collect::<PyResult<Vec<_>>>()
        })
    }
}

impl ToPyObject for TokenizerMode {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            TokenizerMode::Auto => "auto".to_string(),
            TokenizerMode::Slow => "slow".to_string(),
        }
        .into_py(py)
    }
}

impl ToPyObject for Quantization {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            Quantization::Awq => "awg".to_string(),
        }
        .into_py(py)
    }
}

impl TryFrom<Value> for LLMBuilder {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.as_object() {
            Some(map) => {
                let model = map
                    .get("model")
                    .ok_or("Json object must have `model` key")?
                    .as_str()
                    .ok_or("`model` key must be a str")?;

                Ok(LLMBuilder::new(model))
            }
            None => match value.as_str() {
                Some(model) => Ok(LLMBuilder::new(model)),
                None => Err("Json value expected as str or object"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::SamplingParamsBuilder;

    use super::*;

    #[test]
    #[ignore = "requires model download"]
    fn vllm_quickstart() {
        // quickstart example from https://vllm.readthedocs.io/en/latest/getting_started/quickstart.html
        let prompts = [
            "Hello, my name is",
            "The president of the United States is",
            "The capital of France is",
            "The future of AI is",
        ];
        let sampling_params = SamplingParamsBuilder::new()
            .temperature(0.8)
            .top_p(0.95)
            .build()
            .unwrap();

        let llm = LLMBuilder::new("facebook/opt-125m").build().unwrap();
        let outputs = llm.generate(&prompts, Some(&sampling_params)).unwrap();
        assert_eq!(prompts.len(), outputs.len());
    }

    #[test]
    #[ignore = "requires model download"]
    fn model_support() {
        if let Err(e) = LLMBuilder::new("intfloat/e5-small").build() {
            assert!(e.to_string().contains("not supported"));
        }
    }
}
