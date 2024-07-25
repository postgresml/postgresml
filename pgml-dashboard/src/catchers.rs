use rocket::{catch, http::Status, request::Request, response::Redirect};

use crate::responses::{self, BadRequest, Response};

#[catch(403)]
pub async fn not_authorized_catcher(_status: Status, _request: &Request<'_>) -> Redirect {
    Redirect::to("/login")
}

#[catch(404)]
pub async fn not_found_handler(_status: Status, _request: &Request<'_>) -> Response {
    Response::not_found()
}

#[catch(default)]
pub async fn error_catcher(status: Status, request: &Request<'_>) -> Result<BadRequest, responses::Error> {
    Err(responses::Error(anyhow::anyhow!(
        "{} {}\n{:?}",
        status.code,
        status.reason().unwrap(),
        request
    )))
}
