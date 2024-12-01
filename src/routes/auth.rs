use axum::{http::StatusCode, Json};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    email: String,
    username: String,
    password: String,
    name: String,
    gender: Gender,
    dob: (u8, u8, u16),
}

#[derive(Deserialize, Debug)]
enum Gender {
    Male,
    Female,
    Other,
}

pub async fn register(Json(body): Json<RegisterRequest>) -> Result<String, StatusCode> {
    let is_valid = body.username != "" && body.password != "";
    if is_valid {
        match crate::sessions::make_token(body.username.as_str()) {
            Ok(token) => return Ok(token),
            Err(e) => {
                eprintln!("{e}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login(Json(body): Json<LoginRequest>) -> Result<String, StatusCode> {
    let is_valid = body.username != "" && body.password != "";
    if is_valid {
        match crate::sessions::make_token(body.username.as_str()) {
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

#[derive(Deserialize, Debug)]
pub struct LogoutRequest {
    username: String,
}

pub async fn logout(Json(body): Json<LogoutRequest>) {
    println!("{body:?}");
}

pub async fn login_page() {}
pub async fn signup_page() {}
