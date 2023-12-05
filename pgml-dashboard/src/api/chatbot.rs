use anyhow::Context;
use futures::stream::StreamExt;
use pgml::{types::GeneralJsonAsyncIterator, Collection, OpenSourceAI, Pipeline};
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

impl ChatRole {
    fn to_model_specific_role(&self, brain: &ChatbotBrain) -> &'static str {
        match self {
            ChatRole::User => "user",
            ChatRole::Bot => match brain {
                ChatbotBrain::OpenAIGPT4 => "assistant",
                _ => "model",
            },
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum ChatbotBrain {
    OpenAIGPT4,
    TekniumOpenHermes25Mistral7B,
}

impl ChatbotBrain {
    fn is_open_source(&self) -> bool {
        match self {
            Self::OpenAIGPT4 => false,
            _ => true,
        }
    }

    fn get_system_message(
        &self,
        knowledge_base: &KnowledgeBase,
        context: &str,
    ) -> anyhow::Result<serde_json::Value> {
        match self {
            Self::OpenAIGPT4 => {
                let system_prompt = std::env::var("CHATBOT_CHATGPT_SYSTEM_PROMPT")?;
                let system_prompt = system_prompt
                    .replace("{topic}", knowledge_base.topic())
                    .replace("{persona}", "Engineer")
                    .replace("{language}", "English");
                Ok(serde_json::json!({
                    "role": "system",
                    "content": system_prompt
                }))
            }
            _ => Ok(serde_json::json!({
                "role": "system",
                "content": format!(r#"You are a friendly and helpful chatbot that uses the following documents to answer the user's questions with the best of your ability. There is one rule: Do Not Lie.

{}

                "#, context)
            })),
        }
    }
}

impl TryFrom<u8> for ChatbotBrain {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> anyhow::Result<Self> {
        match value {
            0 => Ok(ChatbotBrain::TekniumOpenHermes25Mistral7B),
            1 => Ok(ChatbotBrain::OpenAIGPT4),
            _ => Err(anyhow::anyhow!("Invalid brain id")),
        }
    }
}

impl From<ChatbotBrain> for &'static str {
    fn from(value: ChatbotBrain) -> Self {
        match value {
            ChatbotBrain::OpenAIGPT4 => "gpt-4",
            ChatbotBrain::TekniumOpenHermes25Mistral7B => "teknium/OpenHermes-2.5-Mistral-7B",
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
        text: &str,
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
            text: text.to_string(),
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

// #[post("/chatbot/get-answer", format = "json", data = "<data>")]
// pub async fn chatbot_get_answer(
//     user: User,
//     data: Json<forms::ChatbotPostData>,
// ) -> Result<ResponseOk, Error> {
//     match wrapped_chatbot_get_answer(user, data).await {
//         Ok(response) => Ok(ResponseOk(
//             json!({
//                 "answer": response,
//             })
//             .to_string(),
//         )),
//         Err(error) => {
//             eprintln!("Error: {:?}", error);
//             Ok(ResponseOk(
//                 json!({
//                     "error": error.to_string(),
//                 })
//                 .to_string(),
//             ))
//         }
//     }
// }

struct UpdateHistory {
    collection: Collection,
    user_document: Document,
    model: ChatbotBrain,
    knowledge_base: KnowledgeBase,
}

impl UpdateHistory {
    fn new(
        collection: Collection,
        user_document: Document,
        model: ChatbotBrain,
        knowledge_base: KnowledgeBase,
    ) -> Self {
        Self {
            collection,
            user_document,
            model,
            knowledge_base,
        }
    }

    fn update_history(mut self, chatbot_response: &str) -> anyhow::Result<()> {
        let chatbot_document = Document::new(
            chatbot_response,
            ChatRole::Bot,
            self.user_document.user_id.to_owned(),
            self.model,
            self.knowledge_base,
        );
        let new_history_messages: Vec<pgml::types::Json> = vec![
            serde_json::to_value(self.user_document).unwrap().into(),
            serde_json::to_value(chatbot_document).unwrap().into(),
        ];
        // We do not want to block our return waiting for this to happen
        tokio::spawn(async move {
            self.collection
                .upsert_documents(new_history_messages, None)
                .await
                .expect("Failed to upsert user history");
        });
        Ok(())
    }
}

#[derive(Serialize)]
struct StreamResponse {
    error: Option<String>,
    result: Option<String>,
}

impl StreamResponse {
    fn from_error<E: std::fmt::Display>(error: E) -> Self {
        StreamResponse {
            error: Some(format!("{error}")),
            result: None,
        }
    }

    fn from_result(result: &str) -> Self {
        StreamResponse {
            error: None,
            result: Some(result.to_string()),
        }
    }
}

#[get("/chatbot/get-answer")]
pub async fn chatbot_get_answer(user: User, ws: ws::WebSocket) -> ws::Stream!['static] {
    ws::Stream! { ws =>
        for await message in ws {
            let v = process_message(message, &user).await;
            match v {
                Ok(v) =>
                match v {
                    ProcessMessageResponse::StreamResponse((mut it, update_history)) => {
                        let mut total_text: Vec<String> = Vec::new();
                        while let Some(value) = it.next().await {
                            match value {
                                Ok(v) => {
                                    let v: &str = v.as_array().unwrap()[0].as_str().unwrap();
                                    total_text.push(v.to_string());
                                    yield ws::Message::from(serde_json::to_string(&StreamResponse::from_result(v)).unwrap());
                                },
                                Err(e) => yield ws::Message::from(serde_json::to_string(&StreamResponse::from_error(e)).unwrap())
                            }
                        }
                        update_history.update_history(&total_text.join("")).unwrap();
                    },
                    ProcessMessageResponse::FullResponse(resp) => {
                        yield ws::Message::from(serde_json::to_string(&StreamResponse::from_result(&resp)).unwrap());
                    }
                }
                Err(e) => {
                    yield ws::Message::from(serde_json::to_string(&StreamResponse::from_error(e)).unwrap());
                }
            }
        };
    }
}

enum ProcessMessageResponse {
    StreamResponse((GeneralJsonAsyncIterator, UpdateHistory)),
    FullResponse(String),
}

#[derive(Deserialize)]
struct Message {
    model: u8,
    knowledge_base: u8,
    question: String,
}

async fn process_message(
    message: Result<ws::Message, ws::result::Error>,
    user: &User,
) -> anyhow::Result<ProcessMessageResponse> {
    if let ws::Message::Text(s) = message? {
        let data: Message = serde_json::from_str(&s)?;
        let brain = ChatbotBrain::try_from(data.model)?;
        let knowledge_base = KnowledgeBase::try_from(data.knowledge_base)?;

        // Create it up here so the timestamps that order the conversation are accurate
        let user_document = Document::new(
            &data.question,
            ChatRole::User,
            user.chatbot_session_id.clone(),
            brain,
            knowledge_base,
        );

        let pipeline = Pipeline::new("v1", None, None, None);
        let collection = knowledge_base.collection();
        let collection = Collection::new(
            collection,
            Some(std::env::var("CHATBOT_DATABASE_URL").expect("CHATBOT_DATABASE_URL not set")),
        );
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

        let history_collection = Collection::new(
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
                let chat_role: ChatRole = serde_json::from_value(m["document"]["role"].to_owned())?;
                Ok(serde_json::json!({
                    "role": chat_role.to_model_specific_role(&brain),
                    "content": m["document"]["text"]
                })
                .into())
            })
            .collect::<anyhow::Result<Vec<pgml::types::Json>>>()?;
        let system_message = brain.get_system_message(&knowledge_base, &context)?;
        history.push(system_message.into());
        history.reverse();

        let update_history =
            UpdateHistory::new(history_collection, user_document, brain, knowledge_base);

        if brain.is_open_source() {
            let op = OpenSourceAI::new(Some(
                std::env::var("CHATBOT_DATABASE_URL").expect("CHATBOT_DATABASE_URL not set"),
            ));
            let stream = op
                .chat_completions_create_stream_async(
                    serde_json::to_value::<&str>(brain.into()).unwrap().into(),
                    history,
                    None,
                    None,
                    None,
                    None,
                )
                .await?;
            Ok(ProcessMessageResponse::StreamResponse((
                stream,
                update_history,
            )))
        } else {
            let response = "test".to_string();
            update_history.update_history(&response)?;
            Ok(ProcessMessageResponse::FullResponse(response.to_string()))
        }
    } else {
        Err(anyhow::anyhow!("Error invalid message format"))
    }
}

// pub async fn wrapped_chatbot_get_answer(
//     user: User,
//     data: Json<forms::ChatbotPostData>,
// ) -> Result<String, Error> {
//     let brain = ChatbotBrain::try_from(data.model)?;
//     let knowledge_base = KnowledgeBase::try_from(data.knowledge_base)?;

//     // Create it up here so the timestamps that order the conversation are accurate
//     let user_document = Document::new(
//         data.question.clone(),
//         ChatRole::User,
//         user.chatbot_session_id.clone(),
//         brain,
//         knowledge_base,
//     );

//     let collection = knowledge_base.collection();
//     let collection = Collection::new(
//         collection,
//         Some(std::env::var("CHATBOT_DATABASE_URL").expect("CHATBOT_DATABASE_URL not set")),
//     );

//     let mut history_collection = Collection::new(
//         "ChatHistory",
//         Some(std::env::var("CHATBOT_DATABASE_URL").expect("CHATBOT_DATABASE_URL not set")),
//     );
//     let messages = history_collection
//         .get_documents(Some(
//             json!({
//                "limit": 5,
//                 "order_by": {"timestamp": "desc"},
//                 "filter": {
//                     "metadata": {
//                         "$and" : [
//                             {
//                                 "$or":
//                                 [
//                                     {"role": {"$eq": ChatRole::Bot}},
//                                     {"role": {"$eq": ChatRole::User}}
//                                 ]
//                             },
//                             {
//                                 "user_id": {
//                                     "$eq": user.chatbot_session_id
//                                 }
//                             },
//                             {
//                                 "knowledge_base": {
//                                     "$eq": knowledge_base
//                                 }
//                             },
//                             {
//                                 "model": {
//                                     "$eq": brain
//                                 }
//                             }
//                         ]
//                     }
//                 }

//             })
//             .into(),
//         ))
//         .await?;

//     let mut history = messages
//         .into_iter()
//         .map(|m| {
//             // Can probably remove this clone
//             let chat_role: ChatRole = serde_json::from_value(m["document"]["role"].to_owned())?;
//             if chat_role == ChatRole::Bot {
//                 Ok(format!("Assistant: {}", m["document"]["text"]))
//             } else {
//                 Ok(format!("User: {}", m["document"]["text"]))
//             }
//         })
//         .collect::<anyhow::Result<Vec<String>>>()?;
//     history.reverse();
//     let history = history.join("\n");

//     let pipeline = Pipeline::new("v1", None, None, None);
//     let context = collection
//         .query()
//         .vector_recall(&data.question, &pipeline, Some(json!({
//             "instruction": "Represent the Wikipedia question for retrieving supporting documents: "
//         }).into()))
//         .limit(5)
//         .fetch_all()
//         .await?
//         .into_iter()
//         .map(|(_, context, metadata)| format!("#### Document {}: {}", metadata["id"], context))
//         .collect::<Vec<String>>()
//         .join("\n");

//     let answer =
//         get_openai_chatgpt_answer(knowledge_base, &history, &context, &data.question).await?;

//     let new_history_messages: Vec<pgml::types::Json> = vec![
//         serde_json::to_value(user_document).unwrap().into(),
//         serde_json::to_value(Document::new(
//             answer.clone(),
//             ChatRole::Bot,
//             user.chatbot_session_id.clone(),
//             brain,
//             knowledge_base,
//         ))
//         .unwrap()
//         .into(),
//     ];

//     // We do not want to block our return waiting for this to happen
//     tokio::spawn(async move {
//         history_collection
//             .upsert_documents(new_history_messages, None)
//             .await
//             .expect("Failed to upsert user history");
//     });

//     Ok(answer)
// }

pub fn routes() -> Vec<Route> {
    routes![chatbot_get_answer]
}
