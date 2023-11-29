use anyhow::Context;
use futures::{StreamExt, Stream};
use rust_bridge::{alias, alias_methods};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::{
    types::{GeneralJsonAsyncIterator, Json, GeneralJsonIterator},
    TransformerPipeline, get_or_set_runtime,
};

#[cfg(feature = "python")]
use crate::types::{JsonPython, GeneralJsonAsyncIteratorPython, GeneralJsonIteratorPython};

#[derive(alias, Debug, Clone)]
pub struct OpenSourceAI {
    database_url: Option<String>,
}

fn try_model_nice_name_to_model_name_and_parameters(
    model_name: &str,
) -> Option<(&'static str, Json)> {
    match model_name {
        "mistralai/Mistral-7B-v0.1" => Some((
            "TheBloke/zephyr-7B-beta-GPTQ",
            serde_json::json!({
              "task": "conversational",
              "model": "TheBloke/zephyr-7B-beta-GPTQ",
              "device_map": "auto",
              "revision": "main",
              "model_type": "mistral"
            })
            .into(),
        )),
        "Llama-2-7b-chat-hf" => Some((
            "TheBloke/Llama-2-7B-Chat-GPTQ",
            serde_json::json!({
              "task": "conversational",
              "model": "TheBloke/zephyr-7B-beta-GPTQ",
              "device_map": "auto",
              "revision": "main",
              "model_type": "llama"
            })
            .into(),
        )),
        _ => None,
    }
}

struct AsyncToSyncJsonIterator(std::pin::Pin<Box<dyn Stream<Item = anyhow::Result<Json>> + Send>>);

impl Iterator for AsyncToSyncJsonIterator {
    type Item = anyhow::Result<Json>;

    fn next(&mut self) -> Option<Self::Item> {
        let runtime = get_or_set_runtime();
        runtime.block_on(self.0.next())
    }
}

#[alias_methods(new, chat_completions_create, chat_completions_create_async, chat_completions_create_stream, chat_completions_create_stream_async)]
impl OpenSourceAI {
    pub fn new(database_url: Option<String>) -> Self {
        Self { database_url }
    }

    fn create_pipeline_model_name_parameters(
        &self,
        mut model: Json,
    ) -> anyhow::Result<(TransformerPipeline, String, Json)> {
        if model.is_object() {
            let args = model.as_object_mut().unwrap();
            let model_name = args
                .remove("model")
                .context("`model` is a required key in the model object")?;
            let model_name = model_name.as_str().context("`model` must be a string")?;
            Ok((
                TransformerPipeline::new(
                    "conversational",
                    Some(model_name.to_string()),
                    Some(model.clone()),
                    self.database_url.clone(),
                ),
                model_name.to_string(),
                model,
            ))
        } else {
            let model_name = model
                .as_str()
                .context("`model` must either be a string or an object")?;
            let (real_model_name, parameters) =
                try_model_nice_name_to_model_name_and_parameters(model_name).context(
                    r#"Please select one of the provided models: 
mistralai/Mistral-7B-v0.1
"#,
                )?;
            Ok((
                TransformerPipeline::new(
                    "conversational",
                    Some(real_model_name.to_string()),
                    Some(parameters.clone()),
                    self.database_url.clone(),
                ),
                model_name.to_string(),
                parameters,
            ))
        }
    }

    pub async fn chat_completions_create_stream_async(
        &self,
        model: Json,
        messages: Vec<Json>,
        max_tokens: Option<i32>,
        temperature: Option<f64>,
        n: Option<i32>,
    ) -> anyhow::Result<GeneralJsonAsyncIterator> {
        let (transformer_pipeline, model_name, model_parameters) =
            self.create_pipeline_model_name_parameters(model)?;

        let max_tokens = max_tokens.unwrap_or(1000);
        let temperature = temperature.unwrap_or(0.8);
        let n = n.unwrap_or(1) as usize;
        let to_hash = format!("{}{}{}{}", *model_parameters, max_tokens, temperature, n);
        let md5_digest = md5::compute(to_hash.as_bytes());
        let fingerprint = uuid::Uuid::from_slice(&md5_digest.0)?;

        let messages = serde_json::to_value(messages)?.into();
        let iterator = transformer_pipeline
            .transform_stream(
                messages,
                Some(
                    serde_json::json!({ "max_length": max_tokens, "temperature": temperature, "do_sample": true, "num_return_sequences": n })
                        .into(),
                ),
                Some(1)
            )
            .await?;

        let id = Uuid::new_v4().to_string();
        let iter = iterator.map(move |choices| {
            let since_the_epoch = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            eprintln!("{:?}", choices);
            Ok(serde_json::json!({
                "id": id.clone(),
                "system_fingerprint": fingerprint.clone(),
                "object": "chat.completion.chunk",
                "created": since_the_epoch.as_secs(),
                "model": model_name.clone(),
                "choices": choices?.as_array().context("Error parsing choices from GeneralJsonAsyncIterator")?.iter().enumerate().map(|(i, c)| {
                    serde_json::json!({
                        "index": i,
                        "delta": {
                            "role": "assistant",
                            "content": c
                        }                    
                    })
                    // finish_reason goes here
                }).collect::<serde_json::Value>()
            })
            .into())
        });

        Ok(GeneralJsonAsyncIterator(Box::pin(iter)))
    }

    pub fn chat_completions_create_stream(
        &self,
        model: Json,
        messages: Vec<Json>,
        max_tokens: Option<i32>,
        temperature: Option<f64>,
        n: Option<i32>,
    ) -> anyhow::Result<GeneralJsonIterator> {
        let runtime = crate::get_or_set_runtime();
        let iter = runtime.block_on(self.chat_completions_create_stream_async(model, messages, max_tokens, temperature, n))?;
        Ok(GeneralJsonIterator(Box::new(AsyncToSyncJsonIterator(Box::pin(iter)))))
    }

    pub async fn chat_completions_create_async(
        &self,
        model: Json,
        messages: Vec<Json>,
        max_tokens: Option<i32>,
        temperature: Option<f64>,
        n: Option<i32>,
    ) -> anyhow::Result<Json> {
        let (transformer_pipeline, model_name, model_parameters) =
            self.create_pipeline_model_name_parameters(model)?;

        let max_tokens = max_tokens.unwrap_or(1000);
        let temperature = temperature.unwrap_or(0.8);
        let n = n.unwrap_or(1) as usize;
        let to_hash = format!("{}{}{}{}", *model_parameters, max_tokens, temperature, n);
        let md5_digest = md5::compute(to_hash.as_bytes());
        let fingerprint = uuid::Uuid::from_slice(&md5_digest.0)?;

        let choices = transformer_pipeline
            .transform(
                messages,
                Some(
                    serde_json::json!({ "max_length": max_tokens, "temperature": temperature, "do_sample": true, "num_return_sequences": n })
                        .into(),
                ),
            )
            .await?;
        let choices: Vec<Json> = choices
            .as_array()
            .context("Error parsing return from TransformerPipeline")?
            .iter()
            .enumerate()
            .map(|(i, c)| {
                serde_json::json!({
                    "index": i,
                    "message": {
                        "role": "assistant",
                        "content": c
                    }
                    // Finish reason should be here
                })
                .into()
            })
            .collect();
        let since_the_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Ok(serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "object": "chat.completion",
            "created": since_the_epoch.as_secs(),
            "model": model_name,
            "system_fingerprint": fingerprint,
            "choices": choices,
            "usage": {
                "prompt_tokens": 0,
                "completion_tokens": 0,
                "total_tokens": 0
            }
        })
        .into())
    }

    pub fn chat_completions_create(
        &self,
        model: Json,
        messages: Vec<Json>,
        max_tokens: Option<i32>,
        temperature: Option<f64>,
        n: Option<i32>,
    ) -> anyhow::Result<Json> {
        let runtime = crate::get_or_set_runtime();
        runtime.block_on(self.chat_completions_create_async(
            model,
            messages,
            max_tokens,
            temperature,
            n,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[test]
    fn can_open_source_ai_create() -> anyhow::Result<()> {
        let client = OpenSourceAI::new(None);
        let results = client.chat_completions_create(Json::from_serializable("mistralai/Mistral-7B-v0.1"), vec![
          serde_json::json!({"role": "system", "content": "You are a friendly chatbot who always responds in the style of a pirate"}).into(),
          serde_json::json!({"role": "user", "content": "How many helicopters can a human eat in one sitting?"}).into(),
        ], Some(10), None, Some(3))?;
        assert!(results["choices"].as_array().is_some());
        Ok(())
    }

    #[sqlx::test]
    fn can_open_source_ai_create_async() -> anyhow::Result<()> {
        let client = OpenSourceAI::new(None);
        let results = client.chat_completions_create_async(Json::from_serializable("mistralai/Mistral-7B-v0.1"), vec![
          serde_json::json!({"role": "system", "content": "You are a friendly chatbot who always responds in the style of a pirate"}).into(),
          serde_json::json!({"role": "user", "content": "How many helicopters can a human eat in one sitting?"}).into(),
        ], Some(10), None, Some(3)).await?;
        assert!(results["choices"].as_array().is_some());
        Ok(())
    }

    #[sqlx::test]
    fn can_open_source_ai_create_stream_async() -> anyhow::Result<()> {
        let client = OpenSourceAI::new(None);
        let mut stream = client.chat_completions_create_stream_async(Json::from_serializable("mistralai/Mistral-7B-v0.1"), vec![
          serde_json::json!({"role": "system", "content": "You are a friendly chatbot who always responds in the style of a pirate"}).into(),
          serde_json::json!({"role": "user", "content": "How many helicopters can a human eat in one sitting?"}).into(),
        ], Some(10), None, Some(3)).await?;
        while let Some(o) = stream.next().await {
            o?;
        }
        Ok(())
    }

    #[test]
    fn can_open_source_ai_create_stream() -> anyhow::Result<()> {
        let client = OpenSourceAI::new(None);
        let iterator = client.chat_completions_create_stream(Json::from_serializable("mistralai/Mistral-7B-v0.1"), vec![
          serde_json::json!({"role": "system", "content": "You are a friendly chatbot who always responds in the style of a pirate"}).into(),
          serde_json::json!({"role": "user", "content": "How many helicopters can a human eat in one sitting?"}).into(),
        ], Some(10), None, Some(3))?;
        for o in iterator {
            o?;
        }
        Ok(())
    }

}
