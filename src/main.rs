pub mod api;
pub mod app_state;
pub mod domain;
pub mod infra;

use api::routes::{auth, employees};
use api::swagger::ApiDoc;
use app_state::AppState;
use axum::{routing::{get, post}, Router};
use domain::{config::AppConfig, ports::EmailService};
use envconfig::Envconfig;
use infra::email::{brevo::BrevoSender, smtp::SmtpSender};
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Tracing (Logging)
    // This provides beautifully formatted logs in your terminal.
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Enterprise Employee Hub...");

    // 2. Load Configuration from `.env`
    dotenvy::dotenv().ok();
    let config = AppConfig::init_from_env().unwrap_or_else(|_| {
        tracing::warn!("Failed to load .env, falling back to defaults!");
        AppConfig {
            port: 3000,
            database_url: "sqlite://sqlite.db?mode=rwc".to_string(),
            email_provider: "smtp".to_string(),
            brevo_api_key: "".to_string(),
            brevo_sender_email: "no-reply@enterprise.com".to_string(),
            jwt_secret: "super_secret".to_string(),
        }
    });

    // 3. Setup Database (SQLite)
    tracing::info!("Connecting to Database...");
    let db_pool = SqlitePoolOptions::new()
        .connect(&config.database_url)
        .await?;

    // Auto-create our table if it doesn't exist
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS employees (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            department TEXT NOT NULL,
            active BOOLEAN NOT NULL DEFAULT 1
        );"
    ).execute(&db_pool).await?;

    // 4. Setup Dynamic Dispatch for Email Provider (Polymorphism)
    // Here we decide AT RUNTIME which email service to use based on the config!
    let email_service: Arc<dyn EmailService> = match config.email_provider.as_str() {
        "brevo" => {
            tracing::info!("Using Brevo as Email Provider");
            Arc::new(BrevoSender::new(config.brevo_api_key.clone(), config.brevo_sender_email.clone()))
        }
        _ => {
            tracing::info!("Using standard SMTP as Email Provider");
            Arc::new(SmtpSender::new())
        }
    };

    // 5. Construct Application State
    let state = AppState {
        config: config.clone(),
        db_pool,
        email_service,
        otp_cache: moka::future::Cache::builder()
            .time_to_live(std::time::Duration::from_secs(300)) // 5 minute expiration
            .build(),
    };

    // 6. Generate Swagger Documentation
    let swagger = SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi());

    // 7. Setup CORS (Cross-Origin Resource Sharing)
    // Industry standard is to restrict this to known frontends, but we use permissive for dev.
    let cors = CorsLayer::permissive();

    // 8. Build the Axum Router
    let app = Router::new()
        // Auth Routes
        .route("/auth/request-otp", post(auth::request_otp))
        .route("/auth/verify", post(auth::verify_otp))
        // Employee Routes
        .route("/employees", get(employees::list_employees).post(employees::create_employee))
        .route("/employees/bulk", post(api::routes::bulk::upload_csv))
        // Swagger UI
        .merge(swagger)
        // Add Middleware
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        // Inject State
        .with_state(state);

    // 9. Start the Server
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {}", addr);
    tracing::info!("Swagger UI available at http://localhost:{}/swagger-ui/", config.port);
    
    axum::serve(listener, app).await?;

    Ok(())
}
