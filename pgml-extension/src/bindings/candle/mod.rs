use anyhow::{Context, Result};
use candle_core::Device;
use candle_core::Tensor;
use candle_transformers::generation::LogitsProcessor;
use tokenizers::Tokenizer;

// All of our models
mod quantized_llama;
use quantized_llama::QuantizedLlama;
mod mistral;
use mistral::Mistral;

// New models get added here and in the get_model function
enum ModelType {
    QuantizedLlama,
    Mistral,
}

impl TryFrom<(&str, &str)> for ModelType {
    type Error = anyhow::Error;

    fn try_from(value: (&str, &str)) -> Result<Self> {
        let (model_name, model_type) = value;
        let last_4_of_model_name = &model_name[model_name.len() - 4..];

        match model_type {
            "mistral" => {
                if last_4_of_model_name == "GGUF" {
                    Ok(Self::QuantizedLlama)
                } else {
                    Ok(Self::Mistral)
                }
            }
            _ => Err(anyhow::anyhow!("Model type not supported")),
        }
    }
}

trait Model {
    fn forward(&mut self, tokens: &Tensor) -> Result<Tensor>;
    fn tokenizer(&self) -> &Tokenizer;
}

struct Pipeline {
    model: Box<dyn Model>,
    logits_processor: LogitsProcessor,
    repeat_penalty: f32,
    repeat_last_n: usize,
}

impl Pipeline {
    fn run(&mut self, inputs: Vec<&str>, args: &serde_json::Value) -> Result<Vec<String>> {
        let inputs = inputs[0];
        let max_new_tokens = args["max_new_tokens"].as_u64().unwrap_or(10000);

        let mut all_tokens = Vec::new();

        let mut next_token = {
            let tokens = self
                .model
                .tokenizer()
                .encode(inputs, true)
                .map_err(anyhow::Error::msg)?;
            let token_ids = tokens.get_ids();
            let input = Tensor::new(token_ids, &Device::Cpu)?.unsqueeze(0)?;
            let logits = self.model.forward(&input)?;
            let logits = logits.squeeze(0)?;
            self.logits_processor.sample(&logits)?
        };
        eprintln!("GOT TOKEN {:?}", next_token);
        all_tokens.push(next_token);

        let eos_token = *self
            .model
            .tokenizer()
            .get_vocab(true)
            .get("</s>")
            .context("Error getting eos_token")?;

        for _ in 0..max_new_tokens {
            let input = Tensor::new(&[next_token], &Device::Cpu)?.unsqueeze(0)?;
            let logits = self.model.forward(&input)?;
            let logits = logits.squeeze(0)?;
            let logits = if self.repeat_penalty == 1. {
                logits
            } else {
                let start_at = all_tokens.len().saturating_sub(self.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.repeat_penalty,
                    &all_tokens[start_at..],
                )?
            };
            next_token = self.logits_processor.sample(&logits)?;
            eprintln!("GOT TOKEN {:?}", next_token);
            all_tokens.push(next_token);
            if next_token == eos_token {
                break;
            };
        }
        let output: Vec<String> = all_tokens
            .into_iter()
            .map(|t| token_to_string(t, self.model.tokenizer()))
            .filter(|t| t.is_some())
            .map(|t| t.unwrap())
            .collect();
        eprintln!("{}", output.join(""));
        Ok(output)
    }
}

fn token_to_string(next_token: u32, tokenizer: &Tokenizer) -> Option<String> {
    if let Some(text) = tokenizer.id_to_token(next_token) {
        let text = text.replace('▁', " ");
        let ascii = text
            .strip_prefix("<0x")
            .and_then(|t| t.strip_suffix('>'))
            .and_then(|t| u8::from_str_radix(t, 16).ok());
        match ascii {
            None => Some(text.to_string()),
            Some(ascii) => {
                if let Some(chr) = char::from_u32(ascii as u32) {
                    if chr.is_ascii() {
                        Some(chr.to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    } else {
        None
    }
}

fn get_model(task: &serde_json::Value) -> Result<Pipeline> {
    let model_name = task["model"]
        .as_str()
        .context("Candle transform requires that `model` is set in task")?;
    let model_type = task["model_type"]
        .as_str()
        .context("Candle transform requires that `model_type` is set in task")?;

    let model_type = ModelType::try_from((model_name, model_type))?;

    let model: Box<dyn Model> = match model_type {
        ModelType::QuantizedLlama => Box::new(QuantizedLlama::new(task, &Device::Cpu)?),
        ModelType::Mistral => Box::new(Mistral::new(task, &Device::Cpu)?),
    };

    Ok(Pipeline {
        model,
        logits_processor: LogitsProcessor::new(299792458, Some(0.8), Some(0.95)),
        repeat_penalty: 1.1,
        repeat_last_n: 64,
    })
}

pub fn transform(
    task: &serde_json::Value,
    args: &serde_json::Value,
    inputs: Vec<&str>,
) -> Result<serde_json::Value> {
    let mut model = get_model(task)?;
    let output = model.run(inputs, args)?;
    Ok(serde_json::to_value(output)?)
}
