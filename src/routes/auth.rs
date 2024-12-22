use crate::database::DBConf;
use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

pub fn auth_routes(webdb: Arc<DBConf>) -> Router {
    Router::new()
        .route("/api/user/create_user", post(create_user))
        .route("/api/user/verify_email", post(verify_email))
        .route("/api/user/set_password", post(set_password))
        .route("/api/user/register", post(register))
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
    State(state): State<Arc<DBConf>>,
    ConnectInfo(conn_info): ConnectInfo<crate::utils::ClientConnInfo>,
    Json(body): Json<CreateUserRequest>,
) -> Result<String, (StatusCode, String)> {
    state
        .create_user(body.name, body.email, body.day, body.month, body.year)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    todo!()
}

/// second step of registering an user

#[derive(Deserialize)]
pub struct VerifyEmailRequest {
    email: String,
    otp: u32,
}

pub async fn verify_email(
    State(state): State<Arc<DBConf>>,
    ConnectInfo(conn_info): ConnectInfo<crate::utils::ClientConnInfo>,
    Json(body): Json<VerifyEmailRequest>,
) -> Result<String, (StatusCode, String)> {
    todo!()
}

/// third step of registering an user

#[derive(Deserialize)]
pub struct SetPasswordRequest {
    email: String,
    password: String,
}

pub async fn set_password(
    State(state): State<Arc<DBConf>>,
    ConnectInfo(conn_info): ConnectInfo<crate::utils::ClientConnInfo>,
    Json(body): Json<SetPasswordRequest>,
) -> Result<String, (StatusCode, String)> {
    todo!()
}

/// last step of registering an user

#[derive(Deserialize)]
pub struct RegisterRequest {
    email: String,
    username: String,
}

pub async fn register(
    State(state): State<Arc<DBConf>>,
    ConnectInfo(conn_info): ConnectInfo<crate::utils::ClientConnInfo>,
    Json(body): Json<RegisterRequest>,
) -> Result<String, (StatusCode, String)> {
    // checking and creating user
    if let Err(e) = state.add_user(&user).await {
        tracing::error!("Failed to create user {e:?}");
        if let Some(s) = e.get_custom::<&str>() {
            return Err((StatusCode::BAD_REQUEST, s.to_string()));
        } else {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create user"),
            ));
        }
    }
    // creating token
    match crate::utils::jwt::generate(user.username.as_str(), conn_info.into_ip()) {
        Ok(token) => return Ok(token),
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    };
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login(
    State(state): State<Arc<DBConf>>,
    ConnectInfo(conn_info): ConnectInfo<crate::utils::ClientConnInfo>,
    Json(body): Json<LoginRequest>,
) -> Result<String, (StatusCode, String)> {
    // validating username and password
    if body.username.len() >= 3 && body.password.len() >= 8 {
        if let Err(e) = state.check_password(&body.username, &body.password).await {
            if let Some(s) = e.get_custom::<&str>() {
                return Err((StatusCode::BAD_REQUEST, s.to_string()));
            } else {
                return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("")));
            }
        }
    }
    // creating token
    match crate::utils::jwt::generate(body.username.as_str(), conn_info.into_ip()) {
        Ok(token) => return Ok(token),
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}
