use rocket::{form::Form, route::Route, serde::json::Json};

use crate::{
    forms,
    guards::Cluster,
    responses::{Error, ResponseOk},
};

#[post("/chatbot", format = "json", data = "<data>")]
pub async fn chatbot_answer(
    cluster: &Cluster,
    data: Json<forms::ChatbotPostData>,
) -> Result<ResponseOk, Error> {
    println!("THE QUESTION: {:?}", data.question);
    Ok(ResponseOk(
        serde_json::json!({
            "answer": "Hello world"
        })
        .to_string(),
    ))
}

pub fn routes() -> Vec<Route> {
    routes![chatbot_answer]
}
