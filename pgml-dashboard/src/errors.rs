use rocket::response::{self, Responder};
use rocket::Request;

#[derive(Debug)]
pub struct Error(pub anyhow::Error);

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Error(error.into())
    }
}

impl<'r> Responder<'r, 'r> for Error {
    fn respond_to(self, request: &Request<'_>) -> response::Result<'r> {
        response::Debug(self.0).respond_to(request)
    }
}
