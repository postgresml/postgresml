use reqwest::{Client, RequestBuilder};
use sqlx::PgConnection;
use std::env;
use tracing::instrument;

use crate::{model::ModelRuntime, models, query_builder, types::Json};

pub fn build_remote_embeddings<'a>(
    source: ModelRuntime,
    model_name: &'a str,
    _model_parameters: Option<&'a Json>,
) -> anyhow::Result<Box<dyn RemoteEmbeddings<'a> + Sync + Send + 'a>> {
    match source {
        // OpenAI endpoint for embedddings does not take any model parameters
        ModelRuntime::OpenAI => Ok(Box::new(OpenAIRemoteEmbeddings::new(model_name))),
        _ => Err(anyhow::anyhow!("Unknown remote embeddings source")),
    }
}

#[async_trait::async_trait]
pub trait RemoteEmbeddings<'a> {
    fn build_request(&self) -> RequestBuilder;
    fn generate_body(&self, text: Vec<String>) -> serde_json::Value;

    #[instrument(skip(self))]
    async fn get_embedding_size(&self) -> anyhow::Result<i64> {
        let response = self.embed(vec!["Hello, World!".to_string()]).await?;
        if response.is_empty() {
            anyhow::bail!("API call to get embedding size returned empty response")
        }
        let embedding_size = response[0].len() as i64;
        Ok(embedding_size)
    }

    #[instrument(skip(self, text))]
    async fn embed(&self, text: Vec<String>) -> anyhow::Result<Vec<Vec<f64>>> {
        let request = self.build_request().json(&self.generate_body(text));
        let response = request.send().await?;

        let response = response.json::<serde_json::Value>().await?;
        self.parse_response(response)
    }

    #[instrument(skip(self))]
    async fn get_chunks(
        &self,
        embeddings_table_name: &str,
        chunks_table_name: &str,
        chunk_ids: Option<&Vec<i64>>,
        connection: &mut PgConnection,
        limit: Option<i64>,
    ) -> anyhow::Result<Vec<models::Chunk>> {
        // Requires _query_text be declared out here so it lives long enough
        let mut _query_text = "".to_string();
        let query = match chunk_ids {
            Some(chunk_ids) => {
                _query_text =
                    query_builder!("SELECT * FROM %s WHERE id = ANY ($1)", chunks_table_name);
                sqlx::query_as(_query_text.as_str())
                    .bind(chunk_ids)
                    .bind(limit)
            }
            None => {
                let limit = limit.unwrap_or(1000);
                _query_text = query_builder!(
                    "SELECT * FROM %s WHERE id NOT IN (SELECT chunk_id FROM %s) LIMIT $1",
                    chunks_table_name,
                    embeddings_table_name
                );
                sqlx::query_as(_query_text.as_str()).bind(limit)
            }
        };

        query
            .fetch_all(connection)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    #[instrument(skip(self, response))]
    fn parse_response(&self, response: serde_json::Value) -> anyhow::Result<Vec<Vec<f64>>> {
        let data = response["data"]
            .as_array()
            .ok_or(anyhow::anyhow!("No data in response"))?;

        let embeddings: Vec<Vec<f64>> = data
            .iter()
            .map(|d| {
                let embedding = d["embedding"]
                    .as_array()
                    .expect("Malformed response from openai. Found while in parse_response");

                embedding
                    .iter()
                    .map(|dd| dd.as_f64().unwrap())
                    .collect::<Vec<f64>>()
            })
            .collect();

        Ok(embeddings)
    }

    #[instrument(skip(self))]
    async fn generate_embeddings(
        &self,
        embeddings_table_name: &str,
        chunks_table_name: &str,
        mut chunk_ids: Option<&Vec<i64>>,
        connection: &mut PgConnection,
    ) -> anyhow::Result<()> {
        loop {
            let chunks = self
                .get_chunks(
                    embeddings_table_name,
                    chunks_table_name,
                    chunk_ids,
                    connection,
                    None,
                )
                .await?;
            if chunks.is_empty() {
                break;
            }
            let (retrieved_chunk_ids, chunk_texts): (Vec<i64>, Vec<String>) = chunks
                .into_iter()
                .map(|chunk| (chunk.id, chunk.chunk))
                .unzip();
            let embeddings = self.embed(chunk_texts).await?;

            let query_string_values = (0..embeddings.len())
                .map(|i| query_builder!("($%d, $%d)", i * 2 + 1, i * 2 + 2))
                .collect::<Vec<String>>()
                .join(",");
            let query_string = format!(
                "INSERT INTO %s (chunk_id, embedding) VALUES {} ON CONFLICT (chunk_id) DO UPDATE SET embedding = EXCLUDED.embedding",
                query_string_values
            );

            let query = query_builder!(query_string, embeddings_table_name);
            let mut query = sqlx::query(&query);

            for i in 0..embeddings.len() {
                query = query.bind(retrieved_chunk_ids[i]).bind(&embeddings[i]);
            }

            query.execute(&mut *connection).await?;

            // Set it to none so if it is not None, we don't just retrived the same chunks over and over
            chunk_ids = None;
        }
        Ok(())
    }
}

pub struct OpenAIRemoteEmbeddings<'a> {
    model_name: &'a str,
}

impl<'a> OpenAIRemoteEmbeddings<'a> {
    fn new(model_name: &'a str) -> Self {
        OpenAIRemoteEmbeddings { model_name }
    }
}

impl<'a> RemoteEmbeddings<'a> for OpenAIRemoteEmbeddings<'a> {
    fn build_request(&self) -> RequestBuilder {
        let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY is not set");
        Client::new()
            .post("https://api.openai.com/v1/embeddings")
            .bearer_auth(openai_api_key)
    }

    fn generate_body(&self, text: Vec<String>) -> serde_json::Value {
        serde_json::json!({
            "model": self.model_name,
            "input": text
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn openai_remote_embeddings() -> anyhow::Result<()> {
        let params = serde_json::json!({}).into();
        let openai_remote_embeddings = build_remote_embeddings(
            ModelRuntime::OpenAI,
            "text-embedding-ada-002",
            Some(&params),
        )?;
        let embedding_size = openai_remote_embeddings.get_embedding_size().await?;
        assert!(embedding_size > 0);
        Ok(())
    }
}
