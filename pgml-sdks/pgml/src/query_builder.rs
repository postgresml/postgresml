// NOTE: DEPRECATED
// This whole file is legacy and is only here to be backwards compatible with collection.query()
// No new things should be added here, instead add new items to collection.vector_search

use anyhow::Context;
use serde_json::json;
use tracing::instrument;

use crate::{pipeline::Pipeline, types::Json, Collection};

#[cfg(feature = "rust_bridge")]
use rust_bridge::{alias, alias_methods};

#[cfg(feature = "python")]
use crate::{pipeline::PipelinePython, types::JsonPython};

#[cfg(feature = "c")]
use crate::{languages::c::JsonC, pipeline::PipelineC};

#[cfg_attr(feature = "rust_bridge", derive(alias))]
#[derive(Clone, Debug)]
pub struct QueryBuilder {
    collection: Collection,
    query: Json,
    pipeline: Option<Pipeline>,
}

#[cfg_attr(
    feature = "rust_bridge",
    alias_methods(limit, filter, vector_recall, to_full_string, fetch_all(skip = "C"))
)]
impl QueryBuilder {
    pub fn new(collection: Collection) -> Self {
        let query = json!({
            "query": {
                "fields": {
                    "text": {

                    }
                }
            }
        })
        .into();
        Self {
            collection,
            query,
            pipeline: None,
        }
    }

    #[instrument(skip(self))]
    pub fn limit(mut self, limit: u64) -> Self {
        self.query["limit"] = json!(limit);
        self
    }

    #[instrument(skip(self))]
    pub fn filter(mut self, mut filter: Json) -> Self {
        let filter = filter
            .0
            .as_object_mut()
            .expect("Filter must be a Json object");
        if let Some(f) = filter.remove("metadata") {
            self.query["query"]["filter"] = f;
        }
        if let Some(mut f) = filter.remove("full_text") {
            self.query["query"]["fields"]["text"]["full_text_filter"] =
                std::mem::take(&mut f["text"]);
        }
        self
    }

    #[instrument(skip(self))]
    pub fn vector_recall(
        mut self,
        query: &str,
        pipeline: &Pipeline,
        query_parameters: Option<Json>,
    ) -> Self {
        self.pipeline = Some(pipeline.clone());
        self.query["query"]["fields"]["text"]["query"] = json!(query);
        if let Some(query_parameters) = query_parameters {
            self.query["query"]["fields"]["text"]["parameters"] = query_parameters.0;
        }
        self
    }

    #[instrument(skip(self))]
    pub async fn fetch_all(mut self) -> anyhow::Result<Vec<(f64, String, Json)>> {
        let results = self
            .collection
            .vector_search(
                self.query,
                self.pipeline
                    .as_mut()
                    .context("cannot fetch all without first calling vector_recall")?,
            )
            .await?;
        results
            .into_iter()
            .map(|mut v| {
                Ok((
                    v["score"].as_f64().context("Error converting core")?,
                    v["chunk"]
                        .as_str()
                        .context("Error converting chunk")?
                        .to_string(),
                    std::mem::take(&mut v["document"]).into(),
                ))
            })
            .collect()
    }
}
