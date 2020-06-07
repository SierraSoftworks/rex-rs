use actix_web::{error, http::StatusCode, HttpResponse};
use std::fmt;

#[derive(Debug, Serialize, Deserialize)]
pub struct APIError {
    pub code: u16,
    pub error: String,
    pub message: String,
}

impl APIError {
    pub fn unauthorized() -> Self {
        Self::new(401, "Unauthorized", "You have not provided a valid authentication token. Please authenticate and try again.")
    }

    pub fn new(code: u16, error: &str, message: &str) -> Self {
        Self {
            code: code,
            error: error.to_string(),
            message: message.to_string(),
        }
    }
}

impl error::ResponseError for APIError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .content_type("application/json; charset=utf-8")
            .json(self)
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[HTTP {} {}] {}", self.code, self.error, self.message)
    }
}

impl From<actix::MailboxError> for APIError {
    fn from(err: actix::MailboxError) -> Self {
        error!("We were unable to call an actor: {}", err);

        sentry::capture_event(sentry::protocol::Event {
            message: Some(format!("Failed to send message to Actix Actor: {}", err).into()),
            level: sentry::protocol::Level::Error,
            ..Default::default()
        });

        Self::new(500, "Internal Server Error", "We ran into a problem, this has been reported and will be looked at.")
    }
}