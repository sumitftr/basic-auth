use axum::{extract::State, http::StatusCode, Json};
use mongodb::{
    bson::{oid::ObjectId, DateTime},
    Database,
};
use serde::Deserialize;
use std::sync::Arc;

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

        let date_of_birth = match mongodb::bson::DateTime::builder()
            .year(item.year)
            .month(item.month)
            .day(item.day)
            .build()
        {
            Ok(v) => v,
            Err(_) => return Err("Invalid Date of Birth".to_string()),
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
    State(state): State<Arc<Database>>,
    Json(body): Json<RegisterRequest>,
) -> Result<String, (StatusCode, String)> {
    match crate::models::User::try_from(body) {
        Ok(user) => {
            // checking email availability
            let x = crate::database::is_email_available(state.as_ref(), &user.email).await;
            if let Ok(false) = x {
                return Err((StatusCode::BAD_REQUEST, format!("Email already taken")));
            } else if let Err(e) = x {
                eprintln!("{e}");
                return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("")));
            }
            // checking username availability
            let y = crate::database::is_username_available(state.as_ref(), &user.username).await;
            if let Ok(false) = y {
                return Err((StatusCode::BAD_REQUEST, format!("Username already taken")));
            } else if let Err(e) = y {
                eprintln!("{e}");
                return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("")));
            }
            // creating user
            if let Err(e) = crate::database::add_user(state.as_ref(), &user).await {
                eprintln!("{e}");
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create user"),
                ));
            }
            // creating token
            match crate::sessions::make_token(user.username.as_str()) {
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
