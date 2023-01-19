#[derive(Responder)]
#[response(status = 200, content_type = "text/html")]
pub struct ResponseOk(pub String);

#[derive(Responder)]
#[response(status = 400, content_type = "text/html")]
pub struct BadRequest(pub String);
