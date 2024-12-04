use axum::{http::StatusCode, Json};
use chrono::Utc;
use mongodb::bson::{oid::ObjectId, DateTime};
use serde::Deserialize;
use std::time::SystemTime;

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    name: String,
    email: String,
    gender: String,
    dob: String,
    username: String,
    password: String,
}

#[derive(Debug)]
pub enum RegistrationError {
    NameNotValid,
    EmailAlreadyTaken,
    EmailNotValid,
    InvalidDate,
    UsernameAlreadyTaken,
    UsernameTooShort,
    UsernameNotValid,
    PasswordTooShort,
}

impl std::convert::TryFrom<RegisterRequest> for crate::models::User {
    type Error = RegistrationError;

    fn try_from(item: RegisterRequest) -> Result<Self, Self::Error> {
        if item.username.len() < 4 {
            return Err(RegistrationError::UsernameTooShort);
        }
        if item.password.len() >= 8 {
            return Err(RegistrationError::PasswordTooShort);
        }

        let date_of_birth: SystemTime = chrono::DateTime::parse_from_rfc3339(&item.dob)
            .unwrap()
            .with_timezone(&Utc)
            .into();

        Ok(Self {
            _id: ObjectId::parse_str(&item.username).unwrap(),
            name: item.name,
            email: item.email,
            gender: item.gender.into(),
            dob: DateTime::from(date_of_birth),
            username: item.username,
            password: item.password,
            created: DateTime::now(),
        })
    }
}

pub async fn register(Json(body): Json<RegisterRequest>) -> Result<String, (StatusCode, String)> {
    match crate::models::User::try_from(body) {
        Ok(entry) => {
            match crate::sessions::make_token(entry.username.as_str()) {
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
        Err(e) => Err((StatusCode::BAD_REQUEST, format!("{e:?}"))),
    }
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login(Json(body): Json<LoginRequest>) -> Result<String, StatusCode> {
    let is_valid = body.username != "" && body.password.len() >= 8;
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
pub async fn register_page() {}
