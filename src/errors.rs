use actix_web::{
    error::ResponseError,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::Display;
use log::error;
use migration::DbErr;

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "An internal error occurred. Please try again later.")]
    InternalError,

    #[display(fmt = "{}", _0)]
    BadRequest(String),

    #[display(fmt = "{}", _0)]
    Unauthorized(String),

    #[display(fmt = "Invalid Token")]
    JWTInvalidToken,
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(serde_json::json!({ "error": self.to_string() }).to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            ServiceError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ServiceError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ServiceError::JWTInvalidToken => StatusCode::UNAUTHORIZED,
        }
    }
}

impl From<DbErr> for ServiceError {
    fn from(err: DbErr) -> Self {
        error!("Databse err: {}", err.to_string());

        Self::InternalError
    }
}
