//! Domain Ports (Interfaces/Traits)
//! This file defines the contracts that our Infrastructure layer MUST fulfill.
//! This is the 'D' in SOLID (Dependency Inversion Principle).

use super::errors::AppError;
use async_trait::async_trait;

/// The contract for sending emails.
/// Whether we use Brevo or standard SMTP, the Business logic doesn't care.
/// It only knows it can call `send_otp`.
#[async_trait]
pub trait EmailService: Send + Sync {
    /// Sends a 6-digit OTP code to the provided email address.
    /// Returns `Ok(())` on success, or an `AppError` on failure.
    async fn send_otp(&self, email: &str, otp: &str) -> Result<(), AppError>;
}
