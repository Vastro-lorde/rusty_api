//! Single Source of Truth for all domain entities.
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

/// Represents an Employee in our system.
/// `FromRow` allows sqlx to automatically map database columns to this struct.
/// `ToSchema` tells utoipa to generate Swagger documentation for it.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct Employee {
    /// The unique identifier of the employee
    #[schema(example = 1)]
    pub id: i64,
    /// Full name of the employee
    #[schema(example = "John Doe")]
    pub name: String,
    /// Department the employee works in
    #[schema(example = "Engineering")]
    pub department: String,
    /// Whether the employee is currently active
    #[schema(example = true)]
    pub active: bool,
}

/// The payload required to create a new Employee
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateEmployeePayload {
    /// Full name of the employee
    #[schema(example = "Jane Smith")]
    pub name: String,
    /// Department the employee works in
    #[schema(example = "Sales")]
    pub department: String,
}

/// The payload for requesting an OTP via email.
/// We use a generic mailinator address for testing as requested.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OtpRequestPayload {
    /// The email address to send the OTP to
    #[schema(example = "admin@mailinator.com")]
    pub email: String,
}

/// The payload for verifying an OTP and receiving a JWT.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct OtpVerifyPayload {
    /// The email address the OTP was sent to
    #[schema(example = "admin@mailinator.com")]
    pub email: String,
    /// The 6-digit OTP code
    #[schema(example = "123456")]
    pub otp: String,
}

/// The response returned when an OTP is successfully verified
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthResponse {
    /// The JSON Web Token used for subsequent authenticated requests
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5c...")]
    pub token: String,
}
