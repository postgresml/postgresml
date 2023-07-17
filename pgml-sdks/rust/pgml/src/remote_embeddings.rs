use reqwest::{Client, RequestBuilder};
use sqlx::postgres::PgPool;
use std::env;

use crate::{models, query_builder};

pub fn build_remote_embeddings<'a>(
    source: &'a str,
    model_name: &'a str,
) -> anyhow::Result<Box<dyn RemoteEmbeddings<'a> + Sync + Send + 'a>> {
    match source {
        "openai" => Ok(Box::new(OpenAIRemoteEmbeddings::new(model_name))),
        _ => Err(anyhow::anyhow!("Unknown remote embeddings source")),
    }
}

#[async_trait::async_trait]
pub trait RemoteEmbeddings<'a> {
    fn build_request(&self) -> RequestBuilder;
    fn generate_body(&self, text: Vec<String>) -> serde_json::Value;

    async fn get_embedding_size(&self) -> anyhow::Result<i64> {
        let response = self
            .build_request()
            .json(&self.generate_body(vec!["PostgresML call to get embeddings size".to_string()]))
            .send()
            .await?;

        let response = response.json::<serde_json::Value>().await?;
        let response = self.parse_response(response)?;
        if response.is_empty() {
            anyhow::bail!("API call to get embedding size returned empty response")
        }
        let embedding_size = response[0].len() as i64;
        Ok(embedding_size)
    }

     async fn embed(&self, text: Vec<String>) -> anyhow::Result<Vec<Vec<f64>>> {
        let response = self
            .build_request()
            .json(&self.generate_body(text))
            .send()
            .await?;

        let response = response.json::<serde_json::Value>().await?;
        self.parse_response(response)
    }

    async fn get_chunks(
        &self,
        embeddings_table_name: &str,
        chunks_table_name: &str,
        splitter_id: i64,
        pool: &PgPool,
        limit: Option<i64>,
    ) -> anyhow::Result<Vec<models::Chunk>> {
        let limit = limit.unwrap_or(1000);

        let chunks: Vec<models::Chunk> = sqlx::query_as(&query_builder!(
            "SELECT * FROM %s WHERE splitter_id = $1 AND id NOT IN (SELECT chunk_id FROM %s) LIMIT $2",
            chunks_table_name,
            embeddings_table_name
        ))
        .bind(splitter_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(chunks)
    }

    fn parse_response(&self, response: serde_json::Value) -> anyhow::Result<Vec<Vec<f64>>> {
        let data = response["data"]
            .as_array()
            .ok_or(anyhow::anyhow!("No data in response"))?;

        let embeddings: Vec<Vec<f64>> = data
            .into_iter()
            .map(|d| {
                let embedding = d["embedding"]
                    .as_array()
                    .expect("Malformed response from openai. Found while in parse_response");

                embedding
                    .into_iter()
                    .map(|dd| dd.as_f64().unwrap())
                    .collect::<Vec<f64>>()
            })
            .collect();

        Ok(embeddings)
    }

    async fn generate_embeddings(
        &self,
        embeddings_table_name: &str,
        chunks_table_name: &str,
        splitter_id: i64,
        pool: &PgPool,
    ) -> anyhow::Result<()> {
        loop {
            let chunks = self
                .get_chunks(
                    embeddings_table_name,
                    chunks_table_name,
                    splitter_id,
                    pool,
                    None
                )
                .await?;
            if chunks.is_empty() {
                break;
            }
            let (chunk_ids, chunk_texts): (Vec<i64>, Vec<String>) = chunks
                .into_iter()
                .map(|chunk| (chunk.id, chunk.chunk))
                .unzip();
            let embeddings = self.embed(chunk_texts).await?;

            let query_string_values = (0..embeddings.len())
                .map(|i| format!("(${}, ${})", i * 2 + 1, i * 2 + 2))
                .collect::<Vec<String>>()
                .join(",");
            let query_string = format!(
                "INSERT INTO %s (chunk_id, embedding) VALUES {}",
                query_string_values
            );

            let query = query_builder!(query_string, embeddings_table_name);
            let mut query = sqlx::query(&query);

            for i in 0..embeddings.len() {
                query = query.bind(chunk_ids[i]).bind(&embeddings[i]);
            }

            query.execute(pool).await?;
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
        let openai_remote_embeddings = build_remote_embeddings("openai", "text-embedding-ada-002")?;
        let embedding_size = openai_remote_embeddings.get_embedding_size().await?;
        assert!(embedding_size > 0);
        Ok(())
    }
}
