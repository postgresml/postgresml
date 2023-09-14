use anyhow::Context;
use pgml::{Collection, Pipeline};
use reqwest::Client;
use rocket::{route::Route, serde::json::Json};
use serde_json::json;

use crate::{
    forms,
    guards::Cluster,
    responses::{Error, ResponseOk},
};

async fn get_openai_chatgpt_answer(context: &str, question: &str) -> Result<String, Error> {
    let openai_api_key = std::env::var("OPENAI_API_KEY")?;
    let base_prompt = std::env::var("CHATBOT_CHATGPT_BASE_PROMPT")?;

    let content = base_prompt
        .replace("{context}", context)
        .replace("{question}", question);

    let body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [{"role": "user", "content": content}],
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

#[post("/chatbot", format = "json", data = "<data>")]
pub async fn chatbot_answer(
    cluster: &Cluster,
    data: Json<forms::ChatbotPostData>,
) -> Result<ResponseOk, Error> {
    // Only do this one time
    // let mut collection = Collection::new("knowledge-base-1", None);
    // let model = pgml::Model::new(None, None, None);
    // let splitter = pgml::Splitter::new(None, None);
    // let mut pipeline = Pipeline::new("pipeline-1", Some(model), Some(splitter), None);
    // collection.add_pipeline(&mut pipeline).await?;
    // collection
    //     .upsert_documents(vec![
    //         json!({
    //             "id": "1",
    //             "text": "Text 1"
    //         })
    //         .into(),
    //         json!({
    //             "id": "2",
    //             "text": "Text 2"
    //         })
    //         .into(),
    //     ])
    //     .await?;

    let collection = match data.knowledge_base {
        _ => "knowledge-base-1",
    };
    let collection = Collection::new(collection, None);
    let mut pipeline = Pipeline::new("pipeline-1", None, None, None);
    let context = collection
        .query()
        .vector_recall(&data.question, &mut pipeline, None)
        .limit(10)
        .fetch_all()
        .await?
        .into_iter()
        .map(|(_, context, _)| context)
        .collect::<Vec<String>>()
        .join("\n");

    let answer = match data.model {
        _ => get_openai_chatgpt_answer(&context, &data.question).await,
    }?;

    Ok(ResponseOk(
        json!({
            "answer": answer
        })
        .to_string(),
    ))
}

pub fn routes() -> Vec<Route> {
    routes![chatbot_answer]
}
