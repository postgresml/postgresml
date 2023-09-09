use rocket::{
    http::{ContentType, Header, Status},
    request, response,
};

// A response that doesn't crash and can be returned from any Rocket route.
pub struct Response {
    pub status: Status,
    pub body: Option<String>,
    pub location: Option<String>,
    pub content_type: ContentType,
}

impl Response {
    /// Create new response.
    fn new(status: Status) -> Response {
        Response {
            status,
            body: None,
            location: None,
            content_type: ContentType::new("text", "html"),
        }
    }

    /// Set response body.
    pub fn body(mut self, body: String) -> Response {
        self.body = Some(body);
        self
    }

    /// Set response location.
    fn location(mut self, location: String) -> Response {
        self.location = Some(location);
        self
    }

    /// Create a 303.
    pub fn redirect(to: String) -> Response {
        Self::new(Status::SeeOther).location(to)
    }

    /// Create a 200.
    pub fn ok(body: String) -> Response {
        Self::new(Status::Ok).body(body)
    }

    /// Create a 400.
    pub fn bad_request(body: String) -> Response {
        Self::new(Status::BadRequest).body(body)
    }

    /// Create a 404.
    pub fn not_found() -> Response {
        Self::new(Status::NotFound)
    }

    pub fn content_type(mut self, content_type: ContentType) -> Response {
        self.content_type = content_type;
        self
    }

    pub fn json(body: String) -> Response {
        Self::new(Status::Ok)
            .body(body)
            .content_type(ContentType::new("application", "json"))
    }
}

impl<'r> response::Responder<'r, 'r> for Response {
    fn respond_to(self, request: &request::Request<'_>) -> response::Result<'r> {
    	if self.status == Status::NotFound {
    		return Err(Status::NotFound);
    	}

        let body = match self.body {
            Some(body) => body,
            None => "".to_owned(),
        };

        let mut binding = response::Response::build_from(body.respond_to(request)?);
        let mut response = binding.header(self.content_type);

        if self.location.is_some() {
            response = response.header(Header::new("Location", self.location.unwrap()));
        }

        response.status(self.status).ok()
    }
}

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
