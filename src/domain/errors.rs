//! Custom Application Errors
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// The central Error Enum for our entire application.
/// By implementing `thiserror::Error`, we get automatic `Display` traits.
#[derive(Debug, Error)]
pub enum AppError {
    /// Returned when a requested entity does not exist in the database.
    #[error("Entity not found: {0}")]
    NotFound(String),

    /// Returned when the user provides invalid credentials or an invalid OTP.
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Returned when a database constraint fails or the connection drops.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Returned when we fail to send an email.
    #[error("Email service error: {0}")]
    Email(String),
    
    /// Returned for unexpected, internal server faults.
    #[error("Internal server error: {0}")]
    Internal(String),
}

/// This is where the magic happens!
/// Axum requires handlers to return something that implements `IntoResponse`.
/// By implementing it for our custom `AppError`, we can safely return `Result<T, AppError>`
/// from our controllers, and Axum will automatically format the HTTP response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            // For security, do not leak raw database errors to the client!
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "A database error occurred".to_string(),
            ),
            AppError::Email(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        // Format the error as a JSON object
        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}
