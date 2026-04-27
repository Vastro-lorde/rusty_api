//! Brevo (Sendinblue) implementation of the EmailService trait.
use crate::domain::errors::AppError;
use crate::domain::ports::EmailService;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct BrevoSender {
    api_key: String,
    sender_email: String,
    client: Client,
}

impl BrevoSender {
    pub fn new(api_key: String, sender_email: String) -> Self {
        Self {
            api_key,
            sender_email,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl EmailService for BrevoSender {
    async fn send_otp(&self, email: &str, otp: &str) -> Result<(), AppError> {
        let payload = json!({
            "sender": {"name": "Enterprise API", "email": self.sender_email},
            "to": [{"email": email}],
            "subject": "Your Login OTP",
            "htmlContent": format!("<html><body><h1>Your OTP is: {}</h1></body></html>", otp)
        });

        let response = self.client
            .post("https://api.brevo.com/v3/smtp/email")
            .header("api-key", &self.api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError::Email(format!("Reqwest error: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Email(format!("Brevo rejected the email: {}", error_text)));
        }

        tracing::info!("Sent Brevo email to {}", email);
        Ok(())
    }
}
