use anyhow::{Context, Result};
use candle_core::Tensor;
use candle_core::{DType, Device};
use candle_nn::VarBuilder;
use candle_transformers::models::mistral;
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;

use super::Model;

pub struct Mistral {
    index_pos: usize,
    model: mistral::Model,
    tokenizer: Tokenizer,
}

impl Mistral {
    pub fn new(task: &serde_json::Value, device: &Device) -> Result<Self> {
        println!("Building the mistral model");

        let model_name = task["model"]
            .as_str()
            .context("Candle transform requires that `model` is set in task")?;

        let revision = task["revision"].as_str().unwrap_or("main");

        let api = Api::new()?;
        let repo = api.repo(Repo::with_revision(
            model_name.to_string(),
            RepoType::Model,
            revision.to_string(),
        ));

        let tokenizer =
            Tokenizer::from_file(repo.get("tokenizer.json")?).map_err(anyhow::Error::msg)?;

        let filenames = vec![
            repo.get("pytorch_model-00001-of-00002.safetensors")?,
            repo.get("pytorch_model-00002-of-00002.safetensors")?,
        ];

        println!("GOT everything loading in the model! {:?}", filenames);

        // Change to true if we want to use flash attention
        let config = mistral::Config::config_7b_v0_1(false);

        let dtype = if device.is_cuda() {
            DType::BF16
        } else {
            DType::F32
        };
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&filenames, dtype, &device)? };
        let model = mistral::Model::new(&config, vb)?;

        Ok(Self {
            index_pos: 0,
            model,
            tokenizer,
        })
    }
}

impl Model for Mistral {
    fn forward(&mut self, input: &Tensor) -> Result<Tensor> {
        let output = self.model.forward(input, self.index_pos);
        self.index_pos += input.dims()[1];
        output.map_err(anyhow::Error::msg)
    }

    fn tokenizer(&self) -> &Tokenizer {
        &self.tokenizer
    }
}
