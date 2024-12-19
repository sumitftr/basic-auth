use crate::database::WebDB;
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

impl std::convert::TryFrom<RegisterRequest> for crate::models::User {
    type Error = String;

    fn try_from(mut item: RegisterRequest) -> Result<Self, Self::Error> {
        let name = crate::models::is_name_valid(&item.name)?;
        let email = if crate::models::is_email_valid(&item.email) {
            item.email
        } else {
            return Err("Invalid Email".to_string());
        };
        let username = crate::models::is_username_valid(&item.username).map(|_| item.username)?;
        if item.password.len() < 8 {
            return Err("Password should be of atleast 8 characters".to_string());
        }
        crate::models::into_gender(&mut item.gender);

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
    ConnectInfo(conn_info): ConnectInfo<crate::utils::ClientConnInfo>,
    State(state): State<Arc<WebDB>>,
    Json(body): Json<RegisterRequest>,
) -> Result<String, (StatusCode, String)> {
    match crate::models::User::try_from(body) {
        Ok(user) => {
            // checking and creating user
            if let Err(e) = state.check_and_add_user(&user).await {
                eprintln!("{e}");
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
                Err(e) => {
                    eprintln!("{e}");
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        String::from("Failed to create token"),
                    ));
                }
            };
        }
        Err(e) => return Err((StatusCode::BAD_REQUEST, e)),
    };
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login(
    ConnectInfo(conn_info): ConnectInfo<crate::utils::ClientConnInfo>,
    Json(body): Json<LoginRequest>,
) -> Result<String, StatusCode> {
    let is_valid = body.username != "" && body.password.len() >= 8;
    if is_valid {
        match crate::utils::jwt::make_token(body.username.as_str(), conn_info.ip()) {
            Ok(token) => return Ok(token),
            Err(e) => {
                eprintln!("{e}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn verify_email() {}
