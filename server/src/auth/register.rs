use axum::{
    Json,
    extract::State,
    http::{HeaderValue, StatusCode, header, header::HeaderMap},
    response::IntoResponse,
};
use common::AppError;
use database::Db;
use serde::Deserialize;
use std::sync::Arc;

/// first step of registering an user

#[derive(Deserialize)]
pub struct CreateUserRequest {
    name: String,
    email: String,
    year: i32,
    month: u8,
    day: u8,
}

pub async fn create_user(
    State(state): State<Arc<Db>>,
    Json(body): Json<CreateUserRequest>,
) -> Result<String, AppError> {
    state
        .create_user(body.name, body.email, body.day, body.month, body.year)
        .await?;
    Ok("".to_string())
}

#[derive(Deserialize)]
pub struct ResendOtpRequest {
    email: String,
}

pub async fn resend_otp(
    State(state): State<Arc<Db>>,
    Json(body): Json<ResendOtpRequest>,
) -> Result<String, AppError> {
    state.resend_otp(body.email).await?;
    Ok("The email has been sent".to_string())
}

/// second step of registering an user

#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    email: String,
    otp: u32,
}

pub async fn verify_email(
    State(state): State<Arc<Db>>,
    Json(body): Json<VerifyEmailRequest>,
) -> Result<String, AppError> {
    state.verify_email(body.email, body.otp).await?;
    Ok("Email Verified".to_string())
}

/// third step of registering an user

#[derive(Deserialize)]
pub struct SetPasswordRequest {
    email: String,
    password: String,
}

pub async fn set_password(
    State(state): State<Arc<Db>>,
    Json(body): Json<SetPasswordRequest>,
) -> Result<String, AppError> {
    state.set_password(body.email, body.password)?;
    Ok("Password has been set".to_string())
}

/// last step of registering an user

#[derive(Deserialize)]
pub struct SetUsernameRequest {
    email: String,
    username: String,
}

pub async fn set_username(
    State(state): State<Arc<Db>>,
    headers: HeaderMap,
    Json(body): Json<SetUsernameRequest>,
) -> Result<impl IntoResponse, AppError> {
    // getting user-agent header
    let user_agent = headers
        .get("User-Agent")
        .map(|v| v.to_str().unwrap_or_default().to_owned())
        .unwrap_or_default();

    // creating session and adding user
    let (user_session, active_user_session, set_cookie_headermap) =
        common::user_session::create_session(user_agent);
    Arc::clone(&state)
        .set_username(body.email, body.username.clone(), &user_session)
        .await?;

    Ok((
        StatusCode::CREATED,
        set_cookie_headermap,
        "User Created".to_string(),
    ))
}
