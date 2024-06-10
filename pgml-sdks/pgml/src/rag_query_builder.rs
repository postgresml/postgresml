use sea_query::{
    Alias, CommonTableExpression, Expr, PostgresQueryBuilder, Query, SimpleExpr, WithClause,
};
use sea_query_binder::{SqlxBinder, SqlxValues};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, FromInto};
use std::collections::HashMap;

use crate::{
    collection::Collection,
    debug_sea_query, models,
    pipeline::Pipeline,
    types::{CustomU64Convertor, IntoTableNameAndSchema, Json},
    vector_search_query_builder::{build_sqlx_query, ValidQuery},
};

const fn default_temperature() -> f32 {
    1.
}
const fn default_max_tokens() -> u64 {
    1000000
}
const fn default_top_p() -> f32 {
    1.
}
const fn default_presence_penalty() -> f32 {
    0.
}

#[allow(dead_code)]
const fn default_n() -> u64 {
    0
}

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

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct ValidCompletion {
    model: String,
    prompt: String,
    #[serde(default = "default_temperature")]
    temperature: f32,
    // Need this when coming from JavaScript as everything is an f64 from JS
    #[serde(default = "default_max_tokens")]
    #[serde_as(as = "FromInto<CustomU64Convertor>")]
    max_tokens: u64,
    #[serde(default = "default_top_p")]
    top_p: f32,
    #[serde(default = "default_presence_penalty")]
    presence_penalty: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct ValidChat {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(default = "default_temperature")]
    temperature: f32,
    // Need this when coming from JavaScript as everything is an f64 from JS
    #[serde(default = "default_max_tokens")]
    #[serde_as(as = "FromInto<CustomU64Convertor>")]
    max_tokens: u64,
    #[serde(default = "default_top_p")]
    top_p: f32,
    #[serde(default = "default_presence_penalty")]
    presence_penalty: f32,
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
    prompt_expr: SimpleExpr,
}

#[derive(Debug, Clone)]
struct FormattedMessage {
    content_expr: SimpleExpr,
    message: ChatMessage,
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
                        content_expr: Expr::cust_with_values("$1", [c.content.clone()]),
                        message: c.clone(),
                    })
                    .collect(),
                chat,
            })),
            (Some(completion), None) => Ok(ValidRAGWrapper::Completion(CompletionRAG {
                prompt_expr: Expr::cust_with_values("$1", [completion.prompt.clone()]),
                completion,
            })),
            (Some(_), Some(_)) => anyhow::bail!("Cannot provide both `completion` and `chat`"),
        }
    }
}

pub async fn build_rag_query(
    query: Json,
    collection: &Collection,
    pipeline: &Pipeline,
    stream: bool,
) -> anyhow::Result<(String, SqlxValues)> {
    let rag: ValidRAG = serde_json::from_value(query.0)?;

    // Convert it to something more convenient to work with
    let mut rag_f: ValidRAGWrapper = rag.clone().try_into()?;

    // Confirm that all variables are uppercase
    if !rag.variables.keys().all(|f| &f.to_uppercase() == f) {
        anyhow::bail!("All variables in RAG query must be uppercase")
    }

    let mut final_query = Query::select();

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

    let mut json_objects = Vec::new();

    for (var_name, var_query) in rag.variables.iter() {
        let (var_replace_select, var_source) = match var_query {
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
                (
                    format!(
                        r#"(SELECT string_agg(chunk, '{}') FROM "{var_name}")"#,
                        vector_search.aggregate.join
                    ),
                    format!(r#"(SELECT json_agg(j) FROM "{var_name}" j)"#),
                )
            }
            ValidVariable::RawSQL(sql) => (format!("({})", sql.sql), format!("({})", sql.sql)),
        };

        if !stream {
            json_objects.push(format!("'{var_name}', {var_source}"));
        }

        match &mut rag_f {
            ValidRAGWrapper::Completion(completion) => {
                completion.prompt_expr = Expr::cust_with_expr(
                    format!("replace($1, '{{{var_name}}}', {var_replace_select})"),
                    completion.prompt_expr.clone(),
                );
            }
            ValidRAGWrapper::Chat(chat) => {
                for message in &mut chat.messages {
                    if message.message.content.contains(&format!("{{{var_name}}}")) {
                        message.content_expr = Expr::cust_with_expr(
                            format!("replace($1, '{{{var_name}}}', {var_replace_select})"),
                            message.content_expr.clone(),
                        )
                    }
                }
            }
        }
    }

    let transform_expr = match rag_f {
        ValidRAGWrapper::Completion(completion) => {
            let mut args = serde_json::json!(completion.completion);
            args.as_object_mut().unwrap().remove("model");
            args.as_object_mut().unwrap().remove("prompt");
            let args_expr = Expr::cust_with_values("$1", [args]);

            let task_expr = Expr::cust_with_values(
                "$1",
                [serde_json::json!({
                    "task": "text-generation",
                    "model": completion.completion.model
                })],
            );

            if stream {
                Expr::cust_with_exprs(
                    "
                        pgml.transform_stream(
                          task   => $1,
                          input  => $2,
                          args   => $3
                        )
                    ",
                    [task_expr, completion.prompt_expr, args_expr],
                )
            } else {
                Expr::cust_with_exprs(
                    "
                        pgml.transform(
                          task   => $1,
                          inputs  => zzzzz_zzzzz_start $2 zzzzz_zzzzz_end,
                          args   => $3
                        ) 
                    ",
                    [task_expr, completion.prompt_expr, args_expr],
                )
            }
        }
        ValidRAGWrapper::Chat(chat) => {
            let mut args = serde_json::json!(chat.chat);
            args.as_object_mut().unwrap().remove("model");
            args.as_object_mut().unwrap().remove("messages");
            let args_expr = Expr::cust_with_values("$1", [args]);

            let task_expr = Expr::cust_with_values(
                "$1",
                [serde_json::json!({
                    "task": "conversational",
                    "model": chat.chat.model
                })],
            );

            let dollar_string = chat
                .messages
                .iter()
                .enumerate()
                .map(|(i, _c)| format!("${}", i + 1))
                .collect::<Vec<String>>()
                .join(", ");
            let prompt_exprs = chat.messages.into_iter().map(|cm| {
                let role_expr = Expr::cust_with_values("$1", [cm.message.role]);
                Expr::cust_with_exprs(
                    "jsonb_build_object('role', $1, 'content', $2)",
                    [role_expr, cm.content_expr],
                )
            });
            let inputs_expr = Expr::cust_with_exprs(dollar_string, prompt_exprs);

            if stream {
                Expr::cust_with_exprs(
                    "
                        pgml.transform_stream(
                          task   => $1,
                          inputs  => zzzzz_zzzzz_start $2 zzzzz_zzzzz_end,
                          args   => $3
                        )
                    ",
                    [task_expr, inputs_expr, args_expr],
                )
            } else {
                Expr::cust_with_exprs(
                    "
                        pgml.transform(
                          task   => $1,
                          inputs  => zzzzz_zzzzz_start $2 zzzzz_zzzzz_end,
                          args   => $3
                        )
                    ",
                    [task_expr, inputs_expr, args_expr],
                )
            }
        }
    };

    if stream {
        final_query.expr(transform_expr);
    } else {
        let sources = format!(",'sources', jsonb_build_object({})", json_objects.join(","));
        final_query.expr(Expr::cust_with_expr(
            format!(
                r#"
                    jsonb_build_object(
                        'rag',
                        $1{sources}
                    )
                "#
            ),
            transform_expr,
        ));
    }

    let (sql, values) = final_query
        .with(with_clause)
        .build_sqlx(PostgresQueryBuilder);

    let sql = sql.replace("zzzzz_zzzzz_start", "ARRAY[");
    let sql = sql.replace("zzzzz_zzzzz_end", "]");

    let sql = if stream {
        format!("DECLARE c CURSOR FOR {sql}")
    } else {
        sql
    };

    debug_sea_query!(RAG, sql, values);

    Ok((sql, values))
}
