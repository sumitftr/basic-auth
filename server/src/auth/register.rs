use axum::{
    Json,
    extract::State,
    http::{
        StatusCode,
        header::{self, HeaderMap},
    },
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
    year: u32,
    month: u8,
    day: u8,
}

pub async fn start(
    State(db): State<Arc<Db>>,
    Json(body): Json<CreateUserRequest>,
) -> Result<String, AppError> {
    db.create_user(body.name, body.email, body.day, body.month, body.year)
        .await?;
    Ok("Your information has been accepted".to_string())
}

#[derive(Deserialize)]
pub struct ResendOtpRequest {
    email: String,
}

pub async fn resend_otp(
    State(db): State<Arc<Db>>,
    Json(body): Json<ResendOtpRequest>,
) -> Result<String, AppError> {
    db.resend_otp(body.email).await?;
    Ok("The email has been sent".to_string())
}

/// second step of registering an user

#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    email: String,
    otp: String,
}

pub async fn verify_email(
    State(db): State<Arc<Db>>,
    Json(body): Json<VerifyEmailRequest>,
) -> Result<String, AppError> {
    db.verify_email(body.email, body.otp).await?;
    Ok("Email Verification successful".to_string())
}

/// third step of registering an user

#[derive(Deserialize)]
pub struct SetPasswordRequest {
    email: String,
    password: String,
}

pub async fn set_password(
    State(db): State<Arc<Db>>,
    Json(body): Json<SetPasswordRequest>,
) -> Result<String, AppError> {
    db.set_password(body.email, body.password)?;
    Ok("Your password has been set".to_string())
}

/// last step of registering an user

#[derive(Deserialize)]
pub struct SetUsernameRequest {
    email: String,
    username: String,
}

pub async fn set_username(
    State(db): State<Arc<Db>>,
    headers: HeaderMap,
    Json(body): Json<SetUsernameRequest>,
) -> Result<impl IntoResponse, AppError> {
    // getting user-agent header
    let user_agent = headers
        .get(header::USER_AGENT)
        .map(|v| v.to_str().unwrap_or_default().to_owned())
        .unwrap_or_default();

    // creating session
    let (user_session, active_user_session, set_cookie_headermap) =
        common::user_session::create_session(user_agent);

    // registering user to primary database
    let user = Arc::clone(&db)
        .set_username(body.email, body.username.clone(), user_session)
        .await?;

    // activating session by adding it to `Db::active`
    db.make_user_active(active_user_session, user);

    Ok((
        StatusCode::CREATED,
        set_cookie_headermap,
        "User Created".to_string(),
    ))
}
