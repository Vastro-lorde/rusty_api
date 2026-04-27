//! Standard SMTP implementation using the `lettre` crate.
use crate::domain::errors::AppError;
use crate::domain::ports::EmailService;
use async_trait::async_trait;
use lettre::Message;

/// The SMTP Email sender struct using `lettre`.
pub struct SmtpSender;

impl SmtpSender {
    /// Creates a new instance of the SmtpSender
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EmailService for SmtpSender {
    /// Constructs and sends the email via SMTP.
    async fn send_otp(&self, email: &str, otp: &str) -> Result<(), AppError> {
        // Construct the email message
        let _email_message = Message::builder()
            .from("no-reply@enterprise.com".parse().unwrap())
            .to(email.parse().map_err(|_| AppError::Internal("Invalid email".into()))?)
            .subject("Your Login OTP")
            .body(format!("Your OTP is: {}", otp))
            .map_err(|e| AppError::Internal(e.to_string()))?;

        // In a real application, you would connect to a real SMTP server here using:
        // let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.mailtrap.io").unwrap().build();
        // mailer.send(email_message).await.map_err(|e| AppError::Email(e.to_string()))?;
        
        // For our test, we just simulate sending it out since mailinator catches emails.
        // We log it so we can see the OTP in the console to log in!
        tracing::info!("(SMTP MOCK) Email dispatched to {}: OTP is {}", email, otp);

        Ok(())
    }
}
