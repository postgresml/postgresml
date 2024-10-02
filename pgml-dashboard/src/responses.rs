use axum::{
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use sentry_anyhow::capture_anyhow;

use crate::{models::User, templates, utils::config};

pub struct ResponseOk(pub String);

impl IntoResponse for ResponseOk {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, self.0)
    }
}

pub struct BadRequest(pub String);

impl IntoResponse for BadRequest {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, self.0)
    }
}

pub struct NotFound(pub String);

impl IntoResponse for NotFound {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::NOT_FOUND, self.0)
    }
}

/// A response that doesn't crash and can be returned from any Rocket route.
pub struct Response {
    pub status: StatusCode,
    pub body: Option<String>,
    pub location: Option<String>,
    pub user: Option<User>,
    pub content_type: String,
    pub no_cache: bool,
}

impl Response {
    /// Create new response.
    fn new(status: StatusCode) -> Response {
        Response {
            status,
            body: None,
            location: None,
            user: None,
            content_type: "text/html".to_string(),
            no_cache: false,
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

    /// 500
    pub fn server_error(body: String) -> Response {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR).body(body)
    }

    /// Create a 303.
    pub fn redirect(to: String) -> Response {
        Self::new(StatusCode::SEE_OTHER).location(to)
    }

    /// Create a 200.
    pub fn ok(body: String) -> Response {
        Self::new(StatusCode::OK).body(body)
    }

    /// Create a 400.
    pub fn bad_request(body: String) -> Response {
        Self::new(StatusCode::BAD_REQUEST).body(body)
    }

    /// Create a 404.
    pub fn not_found() -> Response {
        Self::new(StatusCode::NOT_FOUND)
    }

    /// Set the user on the response, if any.
    pub fn user(mut self, user: User) -> Response {
        self.user = Some(user);
        self
    }

    pub fn content_type(mut self, content_type: String) -> Response {
        self.content_type = content_type;
        self
    }

    pub fn no_cache(mut self) -> Response {
        self.no_cache = true;
        self
    }

    pub fn json(body: String) -> Response {
        Self::new(StatusCode::OK)
            .body(body)
            .content_type("application/json".to_string())
    }

    pub fn turbo_stream(body: String) -> Response {
        Self::new(StatusCode::Ok)
            .body(body)
            .content_type("text/vnd.turbo-stream.html".to_string())
            .no_cache()
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        let body = self.body.unwrap_or_else(|| match self.status {
            StatusCode::NOT_FOUND => {
                templates::Layout::new("Internal Server Error", None).render(templates::NotFound {})
            }
            _ => "".into(),
        });

        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, self.content_type);

        if self.no_cache {
            headers.insert(header::CACHE_CONTROL, "no-store".into());
        }

        if let Some(location) = self.location {
            headers.insert(header::LOCATION, location);
        }

        (headers, body)
    }
}

pub struct Template<T>(pub T)
where
    T: sailfish::TemplateOnce;

impl<T> From<Template<T>> for String
where
    T: sailfish::TemplateOnce,
{
    fn from(template: Template<T>) -> String {
        template.0.render_once().unwrap()
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

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        capture_anyhow(&self.0);

        let error = if config::dev_mode() {
            self.0.to_string()
        } else {
            "".into()
        };

        let body = templates::Layout::new("Internal Server Error", None).render(templates::Error { error });

        (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/html")],
            body,
        )
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
