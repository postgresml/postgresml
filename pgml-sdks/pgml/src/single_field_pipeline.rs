use crate::model::Model;
use crate::splitter::Splitter;
use crate::types::Json;
use crate::Pipeline;

#[cfg(feature = "python")]
use crate::{model::ModelPython, splitter::SplitterPython, types::JsonPython};

#[allow(dead_code)]
fn build_pipeline(
    name: &str,
    model: Option<Model>,
    splitter: Option<Splitter>,
    parameters: Option<Json>,
) -> Pipeline {
    let parameters = parameters.unwrap_or_default();
    let schema = if let Some(model) = model {
        let mut schema = serde_json::json!({
            "text": {
                "semantic_search": {
                    "model": model.name,
                    "parameters": model.parameters,
                    "hnsw": parameters["hnsw"]
                }
            }
        });
        if let Some(splitter) = splitter {
            schema["text"]["splitter"] = serde_json::json!({
                "model": splitter.name,
                "parameters": splitter.parameters
            });
        }
        if parameters["full_text_search"]["active"]
            .as_bool()
            .unwrap_or_default()
        {
            schema["text"]["full_text_search"] = serde_json::json!({
                "configuration": parameters["full_text_search"]["configuration"].as_str().map(|v| v.to_string()).unwrap_or_else(|| "english".to_string())
            });
        }
        Some(schema.into())
    } else {
        None
    };
    Pipeline::new(name, schema).expect("Error converting pipeline into new multifield pipeline")
}

#[cfg(feature = "python")]
#[pyo3::prelude::pyfunction]
#[allow(non_snake_case)] // This doesn't seem to be working
pub fn SingleFieldPipeline(
    name: &str,
    model: Option<ModelPython>,
    splitter: Option<SplitterPython>,
    parameters: Option<JsonPython>,
) -> Pipeline {
    let model = model.map(|m| *m.wrapped);
    let splitter = splitter.map(|s| *s.wrapped);
    let parameters = parameters.map(|p| p.wrapped);
    build_pipeline(name, model, splitter, parameters)
}

#[cfg(feature = "javascript")]
#[allow(non_snake_case)]
pub fn SingleFieldPipeline<'a>(
    mut cx: neon::context::FunctionContext<'a>,
) -> neon::result::JsResult<'a, neon::types::JsValue> {
    use rust_bridge::javascript::{FromJsType, IntoJsResult};
    let name = cx.argument(0)?;
    let name = String::from_js_type(&mut cx, name)?;

    let model = cx.argument_opt(1);
    let model = <Option<crate::model::Model>>::from_option_js_type(&mut cx, model)?;

    let splitter = cx.argument_opt(2);
    let splitter = <Option<crate::splitter::Splitter>>::from_option_js_type(&mut cx, splitter)?;

    let parameters = cx.argument_opt(3);
    let parameters = <Option<crate::types::Json>>::from_option_js_type(&mut cx, parameters)?;

    let pipeline = build_pipeline(&name, model, splitter, parameters);
    let x = crate::pipeline::PipelineJavascript::from(pipeline);
    x.into_js_result(&mut cx)
}

mod tests {
    #[test]
    fn pipeline_to_pipeline() -> anyhow::Result<()> {
        use super::*;
        use serde_json::json;

        let model = Model::new(
            Some("test_model".to_string()),
            Some("pgml".to_string()),
            Some(
                json!({
                    "test_parameter": 10
                })
                .into(),
            ),
        );
        let splitter = Splitter::new(
            Some("test_splitter".to_string()),
            Some(
                json!({
                "test_parameter": 11
                })
                .into(),
            ),
        );
        let parameters = json!({
            "full_text_search": {
                "active": true,
                "configuration": "test_configuration"
            },
            "hnsw": {
                "m": 16,
                "ef_construction": 64
            }
        });
        let pipeline = build_pipeline(
            "test_name",
            Some(model),
            Some(splitter),
            Some(parameters.into()),
        );
        let schema = json!({
            "text": {
                "splitter": {
                    "model": "test_splitter",
                    "parameters": {
                        "test_parameter": 11
                    }
                },
                "semantic_search": {
                    "model": "test_model",
                    "parameters": {
                        "test_parameter": 10
                    },
                    "hnsw": {
                        "m": 16,
                        "ef_construction": 64
                    }
                },
                "full_text_search": {
                    "configuration": "test_configuration"
                }
            }
        });
        assert_eq!(schema, pipeline.schema.unwrap().0);
        Ok(())
    }
}
