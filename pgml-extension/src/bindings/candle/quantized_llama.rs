use anyhow::{Context, Result};
use candle_core::quantized::gguf_file;
use candle_core::{Device, Tensor};
use candle_transformers::models::quantized_llama;
use hf_hub::api::sync::Api;
use std::fs::File;
use std::path::PathBuf;
use tokenizers::Tokenizer;

use super::Model;

fn get_model_path(repo: &str, filename: &str) -> Result<PathBuf> {
    let api = Api::new()?;
    let api = api.model(repo.to_string());
    api.get(filename).map_err(anyhow::Error::msg)
}

fn get_tokenizer_path(repo: &str) -> Result<PathBuf> {
    let api = Api::new()?;
    let api = api.model(repo.to_string());
    api.get("tokenizer.json").map_err(anyhow::Error::msg)
}

fn get_tokenizer_from_model_name(model_name: &str) -> Result<Tokenizer> {
    let tokenizer_repo = match model_name {
        _ => "mistralai/Mistral-7B-v0.1",
    };
    let tokenizer_path = get_tokenizer_path(tokenizer_repo)?;
    Tokenizer::from_file(tokenizer_path).map_err(anyhow::Error::msg)
}

pub struct QuantizedLlama {
    index_pos: usize,
    quantized_model: quantized_llama::ModelWeights,
    tokenizer: Tokenizer,
}

impl QuantizedLlama {
    pub fn new(task: &serde_json::Value, _device: &Device) -> Result<Self> {
        let model_name = task["model"]
            .as_str()
            .context("Candle transform requires that `model` is set in task")?;
        let model_file = task["model_file"]
            .as_str()
            .context("Candle transform requires that `model_file` is set in task")?;
        let model_path = get_model_path(model_name, model_file)?;
        let mut model_file = File::open(&model_path)?;

        let quantized_model = gguf_file::Content::read(&mut model_file)?;
        let quantized_model =
            quantized_llama::ModelWeights::from_gguf(quantized_model, &mut model_file)
                .map_err(anyhow::Error::msg)?;
        let tokenizer = get_tokenizer_from_model_name(model_name)?;
        Ok(Self {
            index_pos: 0,
            quantized_model,
            tokenizer,
        })
    }
}

impl Model for QuantizedLlama {
    fn forward(&mut self, input: &Tensor) -> Result<Tensor> {
        let output = self.quantized_model.forward(input, self.index_pos);
        self.index_pos += input.dims()[1];
        output.map_err(anyhow::Error::msg)
    }

    fn tokenizer(&self) -> &Tokenizer {
        &self.tokenizer
    }
}
