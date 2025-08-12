use axum::{Json, Router, extract::State, routing::post};
use common::AppError;
use database::Db;
use serde::Deserialize;
use std::sync::Arc;

pub(super) fn auth_routes(webdb: Arc<Db>) -> Router {
    Router::new()
        .route("/api/register/create_user", post(create_user))
        .route("/api/register/resend_otp", post(resend_otp))
        .route("/api/register/verify_email", post(verify_email))
        .route("/api/register/set_password", post(set_password))
        .route("/api/register/set_username", post(set_username))
        .route("/api/user/login", post(login))
        .with_state(webdb)
}

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
    Ok(format!(""))
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
    Ok(format!("The email has been sent"))
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
    state.verify_email(body.email, body.otp)?;
    Ok(format!("Email Verified"))
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
    Ok(format!("Password has been set"))
}

/// last step of registering an user

#[derive(Deserialize)]
pub struct SetUsernameRequest {
    email: String,
    username: String,
}

pub async fn set_username(
    State(state): State<Arc<Db>>,
    Json(body): Json<SetUsernameRequest>,
) -> Result<String, AppError> {
    let u = Arc::clone(&state)
        .set_username(body.email, body.username.clone())
        .await?;
    state.add_user(&u).await?;
    // creating token
    match crate::jwt::generate(body.username.as_str()) {
        Ok(token) => return Ok(token),
        Err(e) => return Err(e),
    };
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login(
    State(state): State<Arc<Db>>,
    Json(body): Json<LoginRequest>,
) -> Result<String, AppError> {
    // validating username and password
    if body.username.len() >= 3 && body.password.len() >= 8 {
        state.check_password(&body.username, &body.password).await?;
    }
    // creating token
    match crate::jwt::generate(body.username.as_str()) {
        Ok(token) => return Ok(token),
        Err(e) => return Err(e),
    }
}
