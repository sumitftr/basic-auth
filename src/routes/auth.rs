use crate::{database::WebDB, models::user};
use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use mongodb::bson::{oid::ObjectId, DateTime};
use serde::Deserialize;
use std::sync::Arc;

pub fn auth_routes(webdb: Arc<WebDB>) -> Router {
    Router::new()
        .route("/api/user/register", post(register))
        .route("/api/user/login", post(login))
        .with_state(webdb)
}

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    name: String,
    email: String,
    gender: String,
    year: i32,
    month: u8,
    day: u8,
    username: String,
    password: String,
}

impl std::convert::TryFrom<RegisterRequest> for user::User {
    type Error = String;

    fn try_from(mut item: RegisterRequest) -> Result<Self, Self::Error> {
        let name = user::is_name_valid(&item.name)?;
        let email = if user::is_email_valid(&item.email) {
            item.email
        } else {
            return Err("Invalid Email".to_string());
        };
        let username = user::is_username_valid(&item.username).map(|_| item.username)?;
        if item.password.len() < 8 {
            return Err("Password should be of atleast 8 characters".to_string());
        }
        user::into_gender(&mut item.gender);

        let dob = match DateTime::builder()
            .year(item.year)
            .month(item.month)
            .day(item.day)
            .build()
        {
            Ok(v) if v > DateTime::now() => return Err("Invalid Date of Birth".to_string()),
            Ok(v) => v,
            Err(e) => return Err(e.to_string()),
        };

        Ok(Self {
            _id: ObjectId::new(),
            name,
            email,
            gender: item.gender,
            dob,
            username,
            password: item.password,
            created: DateTime::now(),
            last_login: DateTime::now(),
        })
    }
}

pub async fn register(
    State(state): State<Arc<WebDB>>,
    ConnectInfo(conn_info): ConnectInfo<crate::utils::ClientConnInfo>,
    Json(body): Json<RegisterRequest>,
) -> Result<String, (StatusCode, String)> {
    let user = user::User::try_from(body).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    // checking and creating user
    if let Err(e) = state.check_and_add_user(&user).await {
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
    match crate::utils::jwt::make_token(user.username.as_str(), conn_info.ip()) {
        Ok(token) => return Ok(token),
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    };
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login(
    State(state): State<Arc<WebDB>>,
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
    match crate::utils::jwt::make_token(body.username.as_str(), conn_info.ip()) {
        Ok(token) => return Ok(token),
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e)),
    }
}

pub async fn verify_email() {}
