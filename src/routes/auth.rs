use crate::database::WebDB;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
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
        if item.username.len() < 4 {
            return Err("Username too short".to_string());
        }
        if item.password.len() < 8 {
            return Err("Password too short".to_string());
        }
        crate::models::into_gender(&mut item.gender);

        let Ok(date_of_birth) = DateTime::builder()
            .year(item.year)
            .month(item.month)
            .day(item.day)
            .build()
        else {
            return Err("Invalid Date of Birth".to_string());
        };

        Ok(Self {
            _id: ObjectId::new(),
            name: item.name,
            email: item.email,
            gender: item.gender,
            dob: date_of_birth,
            username: item.username,
            password: item.password,
            created: DateTime::now(),
        })
    }
}

pub async fn register(
    State(state): State<Arc<WebDB>>,
    Json(body): Json<RegisterRequest>,
) -> Result<String, (StatusCode, String)> {
    match crate::models::User::try_from(body) {
        Ok(user) => {
            // creating user
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
            match crate::utils::jwt::make_token(user.username.as_str()) {
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

pub async fn login(Json(body): Json<LoginRequest>) -> Result<String, StatusCode> {
    let is_valid = body.username != "" && body.password.len() >= 8;
    if is_valid {
        match crate::utils::jwt::make_token(body.username.as_str()) {
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
