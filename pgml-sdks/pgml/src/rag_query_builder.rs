use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use sea_query::{Alias, CommonTableExpression, Expr, PostgresQueryBuilder, Query, WithClause};
use sea_query_binder::{SqlxBinder, SqlxValues};

use crate::{
    collection::Collection,
    debug_sea_query, models,
    pipeline::Pipeline,
    types::{IntoTableNameAndSchema, Json},
    vector_search_query_builder::{build_sqlx_query, ValidQuery},
};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct ValidAggregate {
    join: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct VectorSearch {
    vector_search: ValidQuery,
    aggregate: ValidAggregate,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct RawSQL {
    sql: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
enum ValidVariable {
    VectorSearch(VectorSearch),
    RawSQL(RawSQL),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct ValidCompletion {
    model: String,
    prompt: String,
    temperature: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct ValidChat {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ValidRAG {
    completion: Option<ValidCompletion>,
    chat: Option<ValidChat>,
    #[serde(flatten)]
    variables: HashMap<String, ValidVariable>,
}

#[derive(Debug, Clone)]
struct CompletionRAG {
    completion: ValidCompletion,
    is_prompt_formatted: bool,
}

#[derive(Debug, Clone)]
struct FormattedMessage {
    message: ChatMessage,
    is_formatted: bool,
}

#[derive(Debug, Clone)]
struct ChatRAG {
    chat: ValidChat,
    messages: Vec<FormattedMessage>,
}

#[derive(Debug, Clone)]
enum ValidRAGWrapper {
    Completion(CompletionRAG),
    Chat(ChatRAG),
}

impl TryFrom<ValidRAG> for ValidRAGWrapper {
    type Error = anyhow::Error;

    fn try_from(rag: ValidRAG) -> Result<Self, Self::Error> {
        match (rag.completion, rag.chat) {
            (None, None) => anyhow::bail!("Must provide either `completion` or `chat`"),
            (None, Some(chat)) => Ok(ValidRAGWrapper::Chat(ChatRAG {
                messages: chat
                    .messages
                    .iter()
                    .map(|c| FormattedMessage {
                        message: c.clone(),
                        is_formatted: false,
                    })
                    .collect(),
                chat,
            })),
            (Some(completion), None) => Ok(ValidRAGWrapper::Completion(CompletionRAG {
                completion,
                is_prompt_formatted: false,
            })),
            (Some(_), Some(_)) => anyhow::bail!("Cannot provide both `completion` and `chat`"),
        }
    }
}

pub async fn build_rag_query(
    query: Json,
    collection: &Collection,
    pipeline: &Pipeline,
) -> anyhow::Result<(String, SqlxValues)> {
    let rag: ValidRAG = serde_json::from_value(query.0)?;

    // Convert it to something more convenient to work with
    let mut rag_f: ValidRAGWrapper = rag.clone().try_into()?;

    // Confirm that all variables are uppercase
    if !rag.variables.keys().all(|f| &f.to_uppercase() == f) {
        anyhow::bail!("All variables in RAG query must be uppercase")
    }

    let mut with_clause = WithClause::new();
    let pipeline_table = format!("{}.pipelines", collection.name);
    let mut pipeline_cte = Query::select();
    pipeline_cte
        .from(pipeline_table.to_table_tuple())
        .columns([models::PipelineIden::Schema])
        .and_where(Expr::col(models::PipelineIden::Name).eq(&pipeline.name));
    let mut pipeline_cte = CommonTableExpression::from_select(pipeline_cte);
    pipeline_cte.table_name(Alias::new("pipeline"));
    with_clause.cte(pipeline_cte);

    for (var_name, var_query) in rag.variables.iter() {
        let var_replace_select = match var_query {
            ValidVariable::VectorSearch(vector_search) => {
                let (sqlx_select_statement, sqlx_ctes) = build_sqlx_query(
                    serde_json::json!(vector_search.vector_search).into(),
                    collection,
                    pipeline,
                    false,
                    Some(var_name),
                )
                .await?;
                for cte in sqlx_ctes {
                    with_clause.cte(cte);
                }
                let mut sqlx_query = CommonTableExpression::from_select(sqlx_select_statement);
                sqlx_query.table_name(Alias::new(var_name));
                with_clause.cte(sqlx_query);
                format!(
                    r#"(SELECT string_agg(chunk, '{}') FROM "{var_name}")"#,
                    vector_search.aggregate.join
                )
            }
            ValidVariable::RawSQL(_) => todo!(),
        };

        match &mut rag_f {
            ValidRAGWrapper::Completion(completion) => {
                if completion.is_prompt_formatted {
                    completion.completion.prompt = format!(
                        "replace({}, '{{{var_name}}}', {var_replace_select})",
                        completion.completion.prompt
                    );
                } else {
                    completion.completion.prompt = format!(
                        "replace('{}', '{{{var_name}}}', {var_replace_select})",
                        completion.completion.prompt
                    );
                    completion.is_prompt_formatted = true;
                }
            }
            ValidRAGWrapper::Chat(chat) => {
                for message in &mut chat.messages {
                    if message.message.content.contains(&format!("{{{var_name}}}")) {
                        if message.is_formatted {
                            message.message.content = format!(
                                "replace({}, '{{{var_name}}}', {var_replace_select})",
                                message.message.content
                            );
                        } else {
                            message.message.content = format!(
                                "replace('{}', '{{{var_name}}}', {var_replace_select})",
                                message.message.content
                            );
                            message.is_formatted = true;
                        }
                    }
                }
            }
        }
    }

    let mut final_query = Query::select();

    match rag_f {
        ValidRAGWrapper::Completion(completion) => {
            let mut args = serde_json::json!(completion.completion);
            args.as_object_mut().unwrap().remove("model");
            args.as_object_mut().unwrap().remove("prompt");
            let args_string = serde_json::to_string(&args)?;

            final_query.expr(Expr::cust(format!(
                r#"
                    pgml.transform(
                      task   => '{{
                        "task": "text-generation",
                        "model": "{}"
                      }}'::JSONB,
                      inputs  => ARRAY[{}],
                      args   => '{args_string}'::JSONB
                    ) 
                "#,
                completion.completion.model, completion.completion.prompt
            )));
        }
        ValidRAGWrapper::Chat(chat) => {
            let mut args = serde_json::json!(chat.chat);
            args.as_object_mut().unwrap().remove("model");
            args.as_object_mut().unwrap().remove("messages");
            let args_string = serde_json::to_string(&args)?;
            let prompt: Vec<String> = chat
                .messages
                .into_iter()
                .map(|p| {
                    if p.is_formatted {
                        format!(
                            "jsonb_build_object('role', '{}', 'content', {})",
                            p.message.role, p.message.content
                        )
                    } else {
                        format!(
                            "jsonb_build_object('role', '{}', 'content', '{}')",
                            p.message.role, p.message.content
                        )
                    }
                })
                .collect();
            let prompt: String = prompt.join(",");

            final_query.expr(Expr::cust(format!(
                r#"
                    pgml.transform(
                      task   => '{{
                        "task": "conversational",
                        "model": "{}"
                      }}'::JSONB,
                      inputs  => ARRAY[{}],
                      args   => '{args_string}'::JSONB
                    ) 
                "#,
                chat.chat.model, prompt
            )));
        }
    }

    let (sql, values) = final_query
        .with(with_clause)
        .build_sqlx(PostgresQueryBuilder);
    debug_sea_query!(VECTOR_SEARCH, sql, values);

    Ok((sql, values))
}
