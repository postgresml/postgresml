use pyo3::{prelude::*, types::PyDict};

#[derive(Debug, Clone)]
pub struct SamplingParamsBuilder {
    n: usize,
    best_of: Option<usize>,
    presence_penalty: f64,
    frequency_penalty: f64,
    temperature: f64,
    top_p: f64,
    top_k: i32,
    use_beam_search: bool,
    length_penalty: f64,
    early_stopping: EarlyStopping,
    stop: Option<Vec<String>>,
    stop_token_ids: Option<Vec<i64>>,
    ignore_eos: bool,
    max_tokens: usize,
    logprobs: Option<usize>,
    skip_special_tokens: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EarlyStopping {
    True,
    False,
    Never,
}

pub struct SamplingParams {
    inner: PyObject,
}

impl SamplingParamsBuilder {
    pub fn new() -> Self {
        Self {
            n: 1,
            best_of: None,
            presence_penalty: 0.0,
            frequency_penalty: 0.0,
            temperature: 1.0,
            top_p: 1.0,
            top_k: -1,
            use_beam_search: false,
            length_penalty: 1.0,
            early_stopping: EarlyStopping::False,
            stop: None,
            stop_token_ids: None,
            ignore_eos: false,
            max_tokens: 16,
            logprobs: None,
            skip_special_tokens: true,
        }
    }

    /// Number of output sequences to return for the given prompt.
    pub fn n(mut self, n: usize) -> Self {
        self.n = n;
        self
    }

    /// Number of output sequences that are generated from the prompt.
    /// From these `best_of` sequences, the top `n` sequences are returned.
    /// `best_of` must be greater than or equal to `n`. This is treated as
    /// the beam width when `use_beam_search` is true. By default, `best_of`
    /// is set to `n`.
    pub fn best_of(mut self, best_of: usize) -> Self {
        self.best_of = Some(best_of);
        self
    }

    /// Float that penalizes new tokens based on whether they
    /// appear in the generated text so far. Values > 0 encourage the model
    /// to use new tokens, while values < 0 encourage the model to repeat
    /// tokens.
    pub fn presence_penalty(mut self, presence_penalty: f64) -> Self {
        self.presence_penalty = presence_penalty;
        self
    }

    /// Float that penalizes new tokens based on their
    /// frequency in the generated text so far. Values > 0 encourage the
    /// model to use new tokens, while values < 0 encourage the model to
    /// repeat tokens.
    pub fn frequency_penalty(mut self, frequency_penalty: f64) -> Self {
        self.frequency_penalty = frequency_penalty;
        self
    }

    /// Float that controls the randomness of the sampling. Lower
    /// values make the model more deterministic, while higher values make
    /// the model more random. Zero means greedy sampling.
    pub fn temperature(mut self, temperature: f64) -> Self {
        self.temperature = temperature;
        self
    }

    /// Float that controls the cumulative probability of the top tokens
    /// to consider. Must be in (0, 1]. Set to 1 to consider all tokens.
    pub fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = top_p;
        self
    }

    /// Integer that controls the number of top tokens to consider. Set
    /// to -1 to consider all tokens.
    pub fn top_k(mut self, top_k: i32) -> Self {
        self.top_k = top_k;
        self
    }

    /// Whether to use beam search instead of sampling.
    pub fn use_beam_search(mut self, use_beam_search: bool) -> Self {
        self.use_beam_search = use_beam_search;
        self
    }

    /// Float that penalizes sequences based on their length.
    /// Used in beam search.
    pub fn length_penalty(mut self, length_penalty: f64) -> Self {
        self.length_penalty = length_penalty;
        self
    }

    /// Controls the stopping condition for beam search. It
    /// accepts the following values: `true`, where the generation stops as
    /// soon as there are `best_of` complete candidates; `false`, where an
    /// heuristic is applied and the generation stops when is it very
    /// unlikely to find better candidates; `"never"`, where the beam search
    /// procedure only stops when there cannot be better candidates
    /// (canonical beam search algorithm).
    pub fn early_stopping(mut self, early_stopping: EarlyStopping) -> Self {
        self.early_stopping = early_stopping;
        self
    }

    ///  List of [`String`]s that stop the generation when they are generated.
    /// The returned output will not contain the stop [`String`]s.
    pub fn stop(mut self, stop: &[&str]) -> Self {
        self.stop = Some(stop.iter().map(|s| s.to_string()).collect());
        self
    }

    /// List of tokens that stop the generation when they are
    /// generated. The returned output will contain the stop tokens unless
    /// the stop tokens are sepcial tokens.
    pub fn stop_token_ids(mut self, stop_token_ids: Vec<i64>) -> Self {
        self.stop_token_ids = Some(stop_token_ids);
        self
    }

    /// Whether to ignore the EOS token and continue generating
    /// tokens after the EOS token is generated.
    pub fn ignore_eos(mut self, ignore_eos: bool) -> Self {
        self.ignore_eos = ignore_eos;
        self
    }

    /// Maximum number of tokens to generate per output sequence.
    pub fn max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Number of log probabilities to return per output token.
    pub fn logprobs(mut self, logprobs: usize) -> Self {
        self.logprobs = Some(logprobs);
        self
    }

    /// Whether to skip special tokens in the output.
    /// Defaults to true.
    pub fn skip_special_tokens(mut self, skip_special_tokens: bool) -> Self {
        self.skip_special_tokens = skip_special_tokens;
        self
    }

    pub fn build(self) -> PyResult<SamplingParams> {
        let inner = Python::with_gil(|py| -> PyResult<PyObject> {
            let kwargs = PyDict::new(py);
            kwargs.set_item("n", self.n)?;
            kwargs.set_item("best_of", self.best_of)?;
            kwargs.set_item("presence_penalty", self.presence_penalty)?;
            kwargs.set_item("frequency_penalty", self.frequency_penalty)?;
            kwargs.set_item("temperature", self.temperature)?;
            kwargs.set_item("top_p", self.top_p)?;
            kwargs.set_item("top_k", self.top_k)?;
            kwargs.set_item("use_beam_search", self.use_beam_search)?;
            kwargs.set_item("length_penalty", self.length_penalty)?;
            kwargs.set_item("early_stopping", self.early_stopping)?;
            kwargs.set_item("stop", self.stop)?;
            kwargs.set_item("stop_token_ids", self.stop_token_ids)?;
            kwargs.set_item("ignore_eos", self.ignore_eos)?;
            kwargs.set_item("max_tokens", self.max_tokens)?;
            kwargs.set_item("logprobs", self.logprobs)?;
            kwargs.set_item("skip_special_tokens", self.skip_special_tokens)?;

            let vllm = PyModule::import(py, "vllm")?;
            vllm.getattr("SamplingParams")?
                .call((), Some(kwargs))?
                .extract()
        })?;

        Ok(SamplingParams { inner })
    }
}

impl ToPyObject for EarlyStopping {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            EarlyStopping::True => true.into_py(py),
            EarlyStopping::False => false.into_py(py),
            EarlyStopping::Never => "never".into_py(py),
        }
    }
}

impl ToPyObject for SamplingParams {
    fn to_object(&self, _py: Python<'_>) -> PyObject {
        self.inner.clone()
    }
}
