use std::fmt;

use serde::{Deserialize, Serialize};

/// The error shape returned by every Rex API endpoint.
///
/// The UI's error toasts read `message`, so this shape is part of the public
/// contract and changing it is a breaking change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiError {
    pub code: u16,
    pub error: String,
    pub message: String,
}

impl ApiError {
    pub fn new(code: u16, error: &str, message: &str) -> Self {
        Self {
            code,
            error: error.to_string(),
            message: message.to_string(),
        }
    }

    pub fn unauthorized() -> Self {
        Self::new(
            401,
            "Unauthorized",
            "You have not provided a valid authentication token. Please authenticate and try again.",
        )
    }

    pub fn forbidden(message: &str) -> Self {
        Self::new(403, "Forbidden", message)
    }

    pub fn not_found(message: &str) -> Self {
        Self::new(404, "Not Found", message)
    }

    pub fn bad_request(message: &str) -> Self {
        Self::new(400, "Bad Request", message)
    }

    /// The catch-all we return whenever something failed for reasons the caller
    /// can do nothing about. Details go to the logs, never to the client.
    pub fn internal_server_error() -> Self {
        Self::new(
            500,
            "Internal Server Error",
            "We ran into a problem, this has been reported and will be looked at.",
        )
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[HTTP {} {}] {}", self.code, self.error, self.message)
    }
}

impl std::error::Error for ApiError {}

#[cfg(feature = "actix")]
impl actix_web::ResponseError for ApiError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        actix_web::HttpResponse::build(actix_web::ResponseError::status_code(self))
            .content_type("application/json; charset=utf-8")
            .json(self)
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::from_u16(self.code)
            .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}
