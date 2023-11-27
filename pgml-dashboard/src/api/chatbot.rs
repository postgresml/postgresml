use anyhow::Context;
use pgml::{Collection, Pipeline};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::Client;
use rocket::{
    http::Status,
    outcome::IntoOutcome,
    request::{self, FromRequest},
    route::Route,
    serde::json::Json,
    Request,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    forms,
    responses::{Error, ResponseOk},
};

pub struct User {
    chatbot_session_id: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<User, ()> {
        request
            .cookies()
            .get_private("chatbot_session_id")
            .map(|c| User {
                chatbot_session_id: c.value().to_string(),
            })
            .or_forward(Status::Unauthorized)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
enum ChatRole {
    User,
    Bot,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum ChatbotBrain {
    OpenAIGPT4,
    PostgresMLFalcon180b,
    AnthropicClaude,
    MetaLlama2,
}

impl TryFrom<u8> for ChatbotBrain {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(ChatbotBrain::OpenAIGPT4),
            1 => Ok(ChatbotBrain::PostgresMLFalcon180b),
            2 => Ok(ChatbotBrain::AnthropicClaude),
            3 => Ok(ChatbotBrain::MetaLlama2),
            _ => Err(anyhow::anyhow!("Invalid brain id")),
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum KnowledgeBase {
    PostgresML,
    PyTorch,
    Rust,
    PostgreSQL,
}

impl KnowledgeBase {
    // The topic and knowledge base are the same for now but may be different later
    fn topic(&self) -> &'static str {
        match self {
            Self::PostgresML => "PostgresML",
            Self::PyTorch => "PyTorch",
            Self::Rust => "Rust",
            Self::PostgreSQL => "PostgreSQL",
        }
    }

    fn collection(&self) -> &'static str {
        match self {
            Self::PostgresML => "PostgresML",
            Self::PyTorch => "PyTorch",
            Self::Rust => "Rust",
            Self::PostgreSQL => "PostgreSQL",
        }
    }
}

impl TryFrom<u8> for KnowledgeBase {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(KnowledgeBase::PostgresML),
            1 => Ok(KnowledgeBase::PyTorch),
            2 => Ok(KnowledgeBase::Rust),
            3 => Ok(KnowledgeBase::PostgreSQL),
            _ => Err(anyhow::anyhow!("Invalid knowledge base id")),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Document {
    id: String,
    text: String,
    role: ChatRole,
    user_id: String,
    model: ChatbotBrain,
    knowledge_base: KnowledgeBase,
    timestamp: u128,
}

impl Document {
    fn new(
        text: String,
        role: ChatRole,
        user_id: String,
        model: ChatbotBrain,
        knowledge_base: KnowledgeBase,
    ) -> Document {
        let id = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        Document {
            id,
            text,
            role,
            user_id,
            model,
            knowledge_base,
            timestamp,
        }
    }
}

async fn get_openai_chatgpt_answer(
    knowledge_base: KnowledgeBase,
    history: &str,
    context: &str,
    question: &str,
) -> Result<String, Error> {
    let openai_api_key = std::env::var("OPENAI_API_KEY")?;
    let base_prompt = std::env::var("CHATBOT_CHATGPT_BASE_PROMPT")?;
    let system_prompt = std::env::var("CHATBOT_CHATGPT_SYSTEM_PROMPT")?;

    let system_prompt = system_prompt
        .replace("{topic}", knowledge_base.topic())
        .replace("{persona}", "Engineer")
        .replace("{language}", "English");

    let content = base_prompt
        .replace("{history}", history)
        .replace("{context}", context)
        .replace("{question}", question);

    let body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [{"role": "system", "content": system_prompt}, {"role": "user", "content": content}],
        "temperature": 0.7
    });

    let response = Client::new()
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(openai_api_key)
        .json(&body)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let response = response["choices"]
        .as_array()
        .context("No data returned from OpenAI")?[0]["message"]["content"]
        .as_str()
        .context("The reponse content from OpenAI was not a string")?
        .to_string();

    Ok(response)
}

#[post("/chatbot/get-answer", format = "json", data = "<data>")]
pub async fn chatbot_get_answer(
    user: User,
    data: Json<forms::ChatbotPostData>,
) -> Result<ResponseOk, Error> {
    match wrapped_chatbot_get_answer(user, data).await {
        Ok(response) => Ok(ResponseOk(
            json!({
                "answer": response,
            })
            .to_string(),
        )),
        Err(error) => {
            eprintln!("Error: {:?}", error);
            Ok(ResponseOk(
                json!({
                    "error": error.to_string(),
                })
                .to_string(),
            ))
        }
    }
}

pub async fn wrapped_chatbot_get_answer(
    user: User,
    data: Json<forms::ChatbotPostData>,
) -> Result<String, Error> {
    let brain = ChatbotBrain::try_from(data.model)?;
    let knowledge_base = KnowledgeBase::try_from(data.knowledge_base)?;

    // Create it up here so the timestamps that order the conversation are accurate
    let user_document = Document::new(
        data.question.clone(),
        ChatRole::User,
        user.chatbot_session_id.clone(),
        brain,
        knowledge_base,
    );

    let collection = knowledge_base.collection();
    let collection = Collection::new(
        collection,
        Some(std::env::var("CHATBOT_DATABASE_URL").expect("CHATBOT_DATABASE_URL not set")),
    );

    let mut history_collection = Collection::new(
        "ChatHistory",
        Some(std::env::var("CHATBOT_DATABASE_URL").expect("CHATBOT_DATABASE_URL not set")),
    );
    let messages = history_collection
        .get_documents(Some(
            json!({
               "limit": 5,
                "order_by": {"timestamp": "desc"},
                "filter": {
                    "metadata": {
                        "$and" : [
                            {
                                "$or":
                                [
                                    {"role": {"$eq": ChatRole::Bot}},
                                    {"role": {"$eq": ChatRole::User}}
                                ]
                            },
                            {
                                "user_id": {
                                    "$eq": user.chatbot_session_id
                                }
                            },
                            {
                                "knowledge_base": {
                                    "$eq": knowledge_base
                                }
                            },
                            {
                                "model": {
                                    "$eq": brain
                                }
                            }
                        ]
                    }
                }

            })
            .into(),
        ))
        .await?;

    let mut history = messages
        .into_iter()
        .map(|m| {
            // Can probably remove this clone
            let chat_role: ChatRole = serde_json::from_value(m["document"]["role"].to_owned())?;
            if chat_role == ChatRole::Bot {
                Ok(format!("Assistant: {}", m["document"]["text"]))
            } else {
                Ok(format!("User: {}", m["document"]["text"]))
            }
        })
        .collect::<anyhow::Result<Vec<String>>>()?;
    history.reverse();
    let history = history.join("\n");

    let pipeline = Pipeline::new("v1", None, None, None);
    let context = collection
        .query()
        .vector_recall(&data.question, &pipeline, Some(json!({
            "instruction": "Represent the Wikipedia question for retrieving supporting documents: "
        }).into()))
        .limit(5)
        .fetch_all()
        .await?
        .into_iter()
        .map(|(_, context, metadata)| format!("#### Document {}: {}", metadata["id"], context))
        .collect::<Vec<String>>()
        .join("\n");

    let answer =
        get_openai_chatgpt_answer(knowledge_base, &history, &context, &data.question).await?;

    let new_history_messages: Vec<pgml::types::Json> = vec![
        serde_json::to_value(user_document).unwrap().into(),
        serde_json::to_value(Document::new(
            answer.clone(),
            ChatRole::Bot,
            user.chatbot_session_id.clone(),
            brain,
            knowledge_base,
        ))
        .unwrap()
        .into(),
    ];

    // We do not want to block our return waiting for this to happen
    tokio::spawn(async move {
        history_collection
            .upsert_documents(new_history_messages, None)
            .await
            .expect("Failed to upsert user history");
    });

    Ok(answer)
}

pub fn routes() -> Vec<Route> {
    routes![chatbot_get_answer]
}
