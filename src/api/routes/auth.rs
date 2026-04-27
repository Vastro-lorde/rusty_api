//! Authentication endpoints handling OTP generation and verification
use crate::{
    app_state::AppState,
    domain::{
        errors::AppError,
        models::{AuthResponse, OtpRequestPayload, OtpVerifyPayload},
    },
};
use axum::{extract::State, Json};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[utoipa::path(
    post,
    path = "/auth/request-otp",
    tag = "Authentication",
    request_body(content = OtpRequestPayload, description = "The email address to send the OTP to (use @mailinator.com)"),
    responses(
        (status = 200, description = "OTP successfully dispatched")
    )
)]
pub async fn request_otp(
    State(state): State<AppState>,
    Json(payload): Json<OtpRequestPayload>,
) -> Result<&'static str, AppError> {
    // 1. Generate a Pseudo-Random 6-digit OTP
    let otp = format!("{:06}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() % 1000000);

    tracing::info!("Generated Real OTP for {}: {}", payload.email, otp);

    // 2. Save to Cache with a TTL of 5 minutes!
    state.otp_cache.insert(payload.email.clone(), otp.clone()).await;

    // 3. Dispatch via Brevo!
    state.email_service.send_otp(&payload.email, &otp).await?;

    Ok("OTP dispatched successfully")
}

#[utoipa::path(
    post,
    path = "/auth/verify",
    tag = "Authentication",
    request_body(content = OtpVerifyPayload, description = "The email and OTP to verify"),
    responses(
        (status = 200, description = "Successfully authenticated", body = AuthResponse),
        (status = 401, description = "Invalid OTP")
    )
)]
pub async fn verify_otp(
    State(state): State<AppState>,
    Json(payload): Json<OtpVerifyPayload>,
) -> Result<Json<AuthResponse>, AppError> {
    // 1. Check if the OTP exists in the cache for this email
    let cached_otp = state.otp_cache.get(&payload.email).await
        .ok_or_else(|| AppError::Unauthorized("OTP expired or not requested".to_string()))?;

    // 2. Validate it matches
    if payload.otp != cached_otp {
        return Err(AppError::Unauthorized("Invalid OTP".to_string()));
    }

    // 3. Clear from cache so it cannot be reused
    state.otp_cache.remove(&payload.email).await;

    // 4. Generate JWT
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + (60 * 60 * 24);

    let claims = Claims {
        sub: payload.email.clone(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_ref()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to issue token: {}", e)))?;

    Ok(Json(AuthResponse { token }))
}
