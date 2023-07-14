use reqwest::ClientBuilder;
use sqlx::postgres::PgPool;

use crate::{models, query_builder};

#[async_trait::async_trait]
pub trait RemoteEmbeddings<'a> {
    fn new(model_name: &'a str, splitter_id: i64, chunks_table_name: &'a str, pool: PgPool) -> Self;
    fn set_headers(&self, client_builder: ClientBuilder) -> ClientBuilder;
    fn get_endpoint(&self) -> String;
    fn generate_body(&self, text: Vec<String>) -> String;

    fn get_chunks_table_name(&self) -> &'a str;
    fn get_pool(&self) -> &PgPool;
    fn get_splitter_id(&self) -> i64;

    async fn get_chunks(&self, last_id: i64, limit: Option<i64>) -> anyhow::Result<Vec<models::Chunk>> {
        let limit = limit.unwrap_or(1000);

        let chunks: Vec<models::Chunk> = sqlx::query_as(&query_builder!(
            "SELECT * FROM %s WHERE id > ? ORDER BY id ASC LIMIT ?",
            self.get_chunks_table_name()
        ))
        .bind(last_id)
        .bind(limit)
        .fetch_all(self.get_pool())
        .await?;

        Ok(chunks)
    }
}

pub struct OpenAIRemoteEmbeddings<'a> {
    model_name: &'a str,
    splitter_id: i64,
    chunks_table_name: &'a str,
    pool: PgPool,
}

impl<'a> RemoteEmbeddings<'a> for OpenAIRemoteEmbeddings<'a> {
    fn new(model_name: &'a str, splitter_id: i64, chunks_table_name: &'a str, pool: PgPool) -> Self {
        OpenAIRemoteEmbeddings {
            model_name,
            splitter_id,
            chunks_table_name,
            pool,
        }
    }

    fn set_headers(&self, client_builder: ClientBuilder) -> ClientBuilder {
        client_builder
    }

    fn get_endpoint(&self) -> String {
        "https://api.openai.com/v1/embeddings".to_string()
    }

    fn generate_body(&self, text: Vec<String>) -> String {
        let mut body = String::new();
        for t in text {
            body.push_str(&format!("{}\n", t));
        }
        body
    }

    fn get_chunks_table_name(&self) -> &'a str {
        self.chunks_table_name
    }

    fn get_pool(&self) ->  &PgPool {
        &self.pool
    }

    fn get_splitter_id(&self) -> i64 {
        self.splitter_id
    }
}
