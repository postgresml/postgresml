use anyhow::Context;
use indicatif::MultiProgress;
use rust_bridge::{alias, alias_manual, alias_methods};
use serde_json::json;
use sqlx::{Executor, PgConnection, PgPool};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use tokio::join;
use tracing::instrument;

use crate::{
    collection::ProjectInfo,
    get_or_initialize_pool,
    model::{Model, ModelRuntime},
    multi_field_pipeline::MultiFieldPipeline,
    queries, query_builder,
    remote_embeddings::build_remote_embeddings,
    splitter::Splitter,
    types::{DateTime, Json, TryToNumeric},
    utils,
};

#[cfg(feature = "python")]
use crate::{model::ModelPython, splitter::SplitterPython, types::JsonPython};

/// A pipeline that processes documents
/// This has been deprecated in favor of [MultiFieldPipeline]
#[derive(alias, Debug, Clone)]
pub struct Pipeline {
    pub name: String,
    pub model: Option<Model>,
    pub splitter: Option<Splitter>,
    pub parameters: Option<Json>,
}

#[alias_methods(new, get_status, to_dict)]
impl Pipeline {
    /// Creates a new [Pipeline]
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the pipeline
    /// * `model` - The pipeline [Model]
    /// * `splitter` - The pipeline [Splitter]
    /// * `parameters` - The parameters to the pipeline. Defaults to None
    ///
    /// # Example
    ///
    /// ```
    /// use pgml::{Pipeline, Model, Splitter};
    /// let model = Model::new(None, None, None);
    /// let splitter = Splitter::new(None, None);
    /// let pipeline = Pipeline::new("my_splitter", Some(model), Some(splitter), None);
    /// ```
    pub fn new(
        name: &str,
        model: Option<Model>,
        splitter: Option<Splitter>,
        parameters: Option<Json>,
    ) -> MultiFieldPipeline {
        let parameters = parameters.unwrap_or_default();
        let schema = if let Some(model) = model {
            let mut schema = json!({
                "text": {
                    "embed": {
                        "model": model.name,
                        "parameters": model.parameters,
                        "hnsw": parameters["hnsw"]
                    }
                }
            });
            if let Some(splitter) = splitter {
                schema["text"]["splitter"] = json!({
                    "model": splitter.name,
                    "parameters": splitter.parameters
                });
            }
            if parameters["full_text_search"]["active"]
                .as_bool()
                .unwrap_or_default()
            {
                schema["text"]["full_text_search"] = json!({
                    "configuration": parameters["full_text_search"]["configuration"].as_str().map(|v| v.to_string()).unwrap_or_else(|| "english".to_string())
                });
            }
            Some(schema.into())
        } else {
            None
        };
        MultiFieldPipeline::new(name, schema)
            .expect("Error converting pipeline into new multifield pipeline")
    }
}
