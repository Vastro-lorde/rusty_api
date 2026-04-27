//! Application State
use crate::domain::{config::AppConfig, ports::EmailService};
use sqlx::SqlitePool;
use std::sync::Arc;
use moka::future::Cache;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db_pool: SqlitePool,
    pub email_service: Arc<dyn EmailService>,
    /// In-memory cache for storing OTPs with a TTL
    pub otp_cache: Cache<String, String>,
}
