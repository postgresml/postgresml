use anyhow::Context;
use futures::stream::StreamExt;
use pgml::{types::GeneralJsonAsyncIterator, Collection, OpenSourceAI, Pipeline};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::Client;
use rocket::{
    http::{Cookie, CookieJar, Status},
    outcome::IntoOutcome,
    request::{self, FromRequest},
    route::Route,
    serde::json::Json,
    Request,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

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
    System,
    User,
    Bot,
}

impl ChatRole {
    fn to_model_specific_role(&self, brain: &ChatbotBrain) -> &'static str {
        match self {
            ChatRole::User => "user",
            ChatRole::Bot => match brain {
                ChatbotBrain::OpenAIGPT4 | ChatbotBrain::TekniumOpenHermes25Mistral7B | ChatbotBrain::Starling7b => {
                    "assistant"
                }
                ChatbotBrain::GrypheMythoMaxL213b => "model",
            },
            ChatRole::System => "system",
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum ChatbotBrain {
    OpenAIGPT4,
    TekniumOpenHermes25Mistral7B,
    GrypheMythoMaxL213b,
    Starling7b,
}

impl ChatbotBrain {
    fn is_open_source(&self) -> bool {
        !matches!(self, Self::OpenAIGPT4)
    }

    fn get_system_message(&self, knowledge_base: &KnowledgeBase, context: &str) -> anyhow::Result<serde_json::Value> {
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

    fn into_model_json(self) -> serde_json::Value {
        match self {
            Self::TekniumOpenHermes25Mistral7B => serde_json::json!({
                "model": "TheBloke/OpenHermes-2.5-Mistral-7B-GPTQ",
                "revision": "main",
                "device_map": "auto",
                "quantization_config": {
                    "bits": 4,
                    "max_input_length": 10000
                }
            }),
            Self::GrypheMythoMaxL213b => serde_json::json!({
                "model": "TheBloke/MythoMax-L2-13B-GPTQ",
                "revision": "main",
                "device_map": "auto",
                "quantization_config": {
                    "bits": 4,
                    "max_input_length": 10000
                }
            }),
            Self::Starling7b => serde_json::json!({
                "model": "TheBloke/Starling-LM-7B-alpha-GPTQ",
                "revision": "main",
                "device_map": "auto",
                "quantization_config": {
                    "bits": 4,
                    "max_input_length": 10000
                }
            }),
            _ => unimplemented!(),
        }
    }

    fn get_chat_template(&self) -> Option<&'static str> {
        match self {
            Self::TekniumOpenHermes25Mistral7B => Some("{% for message in messages %}{{'<|im_start|>' + message['role'] + '\n' + message['content'] + '<|im_end|>' + '\n'}}{% endfor %}{% if add_generation_prompt %}{{ '<|im_start|>assistant\n' }}{% endif %}"),
            Self::GrypheMythoMaxL213b => Some("{% for message in messages %}\n{% if message['role'] == 'user' %}\n{{ '### Instruction:\n' + message['content'] + '\n'}}\n{% elif message['role'] == 'system' %}\n{{ message['content'] + '\n'}}\n{% elif message['role'] == 'model' %}\n{{ '### Response:>\n'  + message['content'] + eos_token + '\n'}}\n{% endif %}\n{% if loop.last and add_generation_prompt %}\n{{ '### Response:' }}\n{% endif %}\n{% endfor %}"),
            _ => None
        }
    }
}

impl TryFrom<&str> for ChatbotBrain {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> anyhow::Result<Self> {
        match value {
            "teknium/OpenHermes-2.5-Mistral-7B" => Ok(ChatbotBrain::TekniumOpenHermes25Mistral7B),
            "Gryphe/MythoMax-L2-13b" => Ok(ChatbotBrain::GrypheMythoMaxL213b),
            "openai" => Ok(ChatbotBrain::OpenAIGPT4),
            "berkeley-nest/Starling-LM-7B-alpha" => Ok(ChatbotBrain::Starling7b),
            _ => Err(anyhow::anyhow!("Invalid brain id")),
        }
    }
}

impl From<ChatbotBrain> for &'static str {
    fn from(value: ChatbotBrain) -> Self {
        match value {
            ChatbotBrain::TekniumOpenHermes25Mistral7B => "teknium/OpenHermes-2.5-Mistral-7B",
            ChatbotBrain::GrypheMythoMaxL213b => "Gryphe/MythoMax-L2-13b",
            ChatbotBrain::OpenAIGPT4 => "openai",
            ChatbotBrain::Starling7b => "berkeley-nest/Starling-LM-7B-alpha",
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
            Self::PostgresML => "PostgresML_0",
            Self::PyTorch => "PyTorch_0",
            Self::Rust => "Rust_0",
            Self::PostgreSQL => "PostgreSQL_0",
        }
    }
}

impl TryFrom<&str> for KnowledgeBase {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> anyhow::Result<Self> {
        match value {
            "postgresml" => Ok(KnowledgeBase::PostgresML),
            "pytorch" => Ok(KnowledgeBase::PyTorch),
            "rust" => Ok(KnowledgeBase::Rust),
            "postgresql" => Ok(KnowledgeBase::PostgreSQL),
            _ => Err(anyhow::anyhow!("Invalid knowledge base id")),
        }
    }
}

impl From<KnowledgeBase> for &'static str {
    fn from(value: KnowledgeBase) -> Self {
        match value {
            KnowledgeBase::PostgresML => "postgresml",
            KnowledgeBase::PyTorch => "pytorch",
            KnowledgeBase::Rust => "rust",
            KnowledgeBase::PostgreSQL => "postgresql",
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
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
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

async fn get_openai_chatgpt_answer<M: Serialize>(messages: M) -> anyhow::Result<String> {
    let openai_api_key = std::env::var("OPENAI_API_KEY")?;
    let body = json!({
        "model": "gpt-3.5-turbo",
        "messages": messages,
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

    let response = response["choices"].as_array().context("No data returned from OpenAI")?[0]["message"]["content"]
        .as_str()
        .context("The reponse content from OpenAI was not a string")?
        .to_string();

    Ok(response)
}

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
    id: Option<u64>,
    error: Option<String>,
    result: Option<String>,
    partial_result: Option<String>,
}

impl StreamResponse {
    fn from_error<E: std::fmt::Display>(id: Option<u64>, error: E) -> Self {
        StreamResponse {
            id,
            error: Some(format!("{error}")),
            result: None,
            partial_result: None,
        }
    }

    fn from_result(id: u64, result: &str) -> Self {
        StreamResponse {
            id: Some(id),
            error: None,
            result: Some(result.to_string()),
            partial_result: None,
        }
    }

    fn from_partial_result(id: u64, result: &str) -> Self {
        StreamResponse {
            id: Some(id),
            error: None,
            result: None,
            partial_result: Some(result.to_string()),
        }
    }
}

#[get("/chatbot/clear-history")]
pub async fn clear_history(cookies: &CookieJar<'_>) -> Status {
    // let cookie = Cookie::build("chatbot_session_id").path("/");
    let cookie = Cookie::new("chatbot_session_id", "");
    cookies.remove(cookie);
    Status::Ok
}

#[derive(Serialize)]
pub struct GetHistoryResponse {
    result: Option<Vec<HistoryMessage>>,
    error: Option<String>,
}

#[derive(Serialize)]
struct HistoryMessage {
    side: String,
    content: String,
    knowledge_base: String,
    brain: String,
}

#[get("/chatbot/get-history")]
pub async fn chatbot_get_history(user: User) -> Json<GetHistoryResponse> {
    match do_chatbot_get_history(&user, 100).await {
        Ok(messages) => Json(GetHistoryResponse {
            result: Some(messages),
            error: None,
        }),
        Err(e) => Json(GetHistoryResponse {
            result: None,
            error: Some(format!("{e}")),
        }),
    }
}

async fn do_chatbot_get_history(user: &User, limit: usize) -> anyhow::Result<Vec<HistoryMessage>> {
    let history_collection = Collection::new(
        "ChatHistory_0",
        Some(std::env::var("CHATBOT_DATABASE_URL").expect("CHATBOT_DATABASE_URL not set")),
    )?;
    let mut messages = history_collection
        .get_documents(Some(
            json!({
               "limit": limit,
                "order_by": {"timestamp": "desc"},
                "filter": {
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
                        }
                    ]
                }

            })
            .into(),
        ))
        .await?;
    messages.reverse();
    let messages: anyhow::Result<Vec<HistoryMessage>> = messages
        .into_iter()
        .map(|m| {
            let side: String = m["document"]["role"]
                .as_str()
                .context("Error parsing chat role")?
                .to_string()
                .to_lowercase();
            let content: String = m["document"]["text"]
                .as_str()
                .context("Error parsing text")?
                .to_string();
            let model: ChatbotBrain =
                serde_json::from_value(m["document"]["model"].to_owned()).context("Error parsing model")?;
            let model: &str = model.into();
            let knowledge_base: KnowledgeBase = serde_json::from_value(m["document"]["knowledge_base"].to_owned())
                .context("Error parsing knowledge_base")?;
            let knowledge_base: &str = knowledge_base.into();
            Ok(HistoryMessage {
                side,
                content,
                brain: model.to_string(),
                knowledge_base: knowledge_base.to_string(),
            })
        })
        .collect();
    messages
}

#[get("/chatbot/get-answer")]
pub async fn chatbot_get_answer(user: User, ws: ws::WebSocket) -> ws::Stream!['static] {
    ws::Stream! { ws =>
        for await message in ws {
            let v = process_message(message, &user).await;
            match v {
                Ok((v, id)) =>
                match v {
                    ProcessMessageResponse::StreamResponse((mut it, update_history)) => {
                        let mut total_text: Vec<String> = Vec::new();
                        while let Some(value) = it.next().await {
                            match value {
                                Ok(v) => {
                                    let v: &str = v["choices"][0]["delta"]["content"].as_str().unwrap();
                                    total_text.push(v.to_string());
                                    yield ws::Message::from(serde_json::to_string(&StreamResponse::from_partial_result(id, v)).unwrap());
                                },
                                Err(e) => yield ws::Message::from(serde_json::to_string(&StreamResponse::from_error(Some(id), e)).unwrap())
                            }
                        }
                        update_history.update_history(&total_text.join("")).unwrap();
                    },
                    ProcessMessageResponse::FullResponse(resp) => {
                        yield ws::Message::from(serde_json::to_string(&StreamResponse::from_result(id, &resp)).unwrap());
                    }
                }
                Err(e) => {
                    yield ws::Message::from(serde_json::to_string(&StreamResponse::from_error(None, e)).unwrap());
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
    id: u64,
    model: String,
    knowledge_base: String,
    question: String,
}

async fn process_message(
    message: Result<ws::Message, ws::result::Error>,
    user: &User,
) -> anyhow::Result<(ProcessMessageResponse, u64)> {
    if let ws::Message::Text(s) = message? {
        let data: Message = serde_json::from_str(&s)?;
        let brain = ChatbotBrain::try_from(data.model.as_str())?;
        let knowledge_base = KnowledgeBase::try_from(data.knowledge_base.as_str())?;

        let user_document = Document::new(
            &data.question,
            ChatRole::User,
            user.chatbot_session_id.clone(),
            brain,
            knowledge_base,
        );

        let mut pipeline = Pipeline::new("v1", None)?;
        let collection = knowledge_base.collection();
        let mut collection = Collection::new(
            collection,
            Some(std::env::var("CHATBOT_DATABASE_URL").expect("CHATBOT_DATABASE_URL not set")),
        )?;
        let context = collection
            .vector_search(
                serde_json::json!({
                "query": {
                "fields": {
                    "text": {
                        "query": &data.question,
                        "parameters": {
                            "instruction": "Represent the Wikipedia question for retrieving supporting documents: "
                        }
                    },
                }
                }})
                .into(),
                &mut pipeline,
            )
            .await?
            .into_iter()
            .map(|v| format!("\n\n#### Document {}: \n{}\n\n", v["document"]["id"], v["chunk"]))
            .collect::<Vec<String>>()
            .join("");

        let history_collection = Collection::new(
            "ChatHistory_0",
            Some(std::env::var("CHATBOT_DATABASE_URL").expect("CHATBOT_DATABASE_URL not set")),
        )?;
        let mut messages = history_collection
            .get_documents(Some(
                json!({
                   "limit": 5,
                    "order_by": {"timestamp": "desc"},
                    "filter": {
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
                            // This is where we would match on the model if we wanted to
                        ]
                    }

                })
                .into(),
            ))
            .await?;
        messages.reverse();

        let (mut history, _) = messages
            .into_iter()
            .fold((Vec::new(), None), |(mut new_history, role), value| {
                let current_role: ChatRole =
                    serde_json::from_value(value["document"]["role"].to_owned()).expect("Error parsing chat role");
                if let Some(role) = role {
                    if role == current_role {
                        match role {
                            ChatRole::User => new_history.push(
                                serde_json::json!({
                                    "role": ChatRole::Bot.to_model_specific_role(&brain),
                                    "content": "*no response due to error*"
                                })
                                .into(),
                            ),
                            ChatRole::Bot => new_history.push(
                                serde_json::json!({
                                    "role": ChatRole::User.to_model_specific_role(&brain),
                                    "content": "*no response due to error*"
                                })
                                .into(),
                            ),
                            _ => panic!("Too many system messages"),
                        }
                    }
                    let new_message: pgml::types::Json = serde_json::json!({
                        "role": current_role.to_model_specific_role(&brain),
                        "content": value["document"]["text"]
                    })
                    .into();
                    new_history.push(new_message);
                } else if matches!(current_role, ChatRole::User) {
                    let new_message: pgml::types::Json = serde_json::json!({
                        "role": current_role.to_model_specific_role(&brain),
                        "content": value["document"]["text"]
                    })
                    .into();
                    new_history.push(new_message);
                }
                (new_history, Some(current_role))
            });

        let system_message = brain.get_system_message(&knowledge_base, &context)?;
        history.insert(0, system_message.into());

        // Need to make sure we aren't about to add two user messages back to back
        if let Some(message) = history.last() {
            if message["role"].as_str().unwrap() == ChatRole::User.to_model_specific_role(&brain) {
                history.push(
                    serde_json::json!({
                        "role": ChatRole::Bot.to_model_specific_role(&brain),
                        "content": "*no response due to errors*"
                    })
                    .into(),
                );
            }
        }
        history.push(
            serde_json::json!({
                "role": ChatRole::User.to_model_specific_role(&brain),
                "content": data.question
            })
            .into(),
        );

        let update_history = UpdateHistory::new(history_collection, user_document, brain, knowledge_base);

        if brain.is_open_source() {
            let op = OpenSourceAI::new(Some(
                std::env::var("CHATBOT_DATABASE_URL").expect("CHATBOT_DATABASE_URL not set"),
            ));
            let chat_template = brain.get_chat_template();
            let stream = op
                .chat_completions_create_stream_async(
                    brain.into_model_json().into(),
                    history,
                    Some(10000),
                    None,
                    None,
                    chat_template.map(|t| t.to_string()),
                )
                .await?;
            Ok((
                ProcessMessageResponse::StreamResponse((stream, update_history)),
                data.id,
            ))
        } else {
            let response = match brain {
                ChatbotBrain::OpenAIGPT4 => get_openai_chatgpt_answer(history).await?,
                _ => unimplemented!(),
            };
            update_history.update_history(&response)?;
            Ok((ProcessMessageResponse::FullResponse(response), data.id))
        }
    } else {
        Err(anyhow::anyhow!("Error invalid message format"))
    }
}

pub fn routes() -> Vec<Route> {
    routes![chatbot_get_answer, chatbot_get_history, clear_history]
}
