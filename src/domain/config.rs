//! Application Configuration
use envconfig::Envconfig;

#[derive(Envconfig, Clone, Debug)]
pub struct AppConfig {
    #[envconfig(from = "PORT", default = "3000")]
    pub port: u16,

    #[envconfig(from = "DATABASE_URL", default = "sqlite://sqlite.db?mode=rwc")]
    pub database_url: String,

    #[envconfig(from = "EMAIL_PROVIDER", default = "brevo")]
    pub email_provider: String,

    #[envconfig(from = "BREVO_API_KEY", default = "")]
    pub brevo_api_key: String,

    #[envconfig(from = "BREVO_SENDER_EMAIL", default = "no-reply@enterprise.com")]
    pub brevo_sender_email: String,

    #[envconfig(from = "JWT_SECRET", default = "super_secret")]
    pub jwt_secret: String,
}
