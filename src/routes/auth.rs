use axum::Json;
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

pub async fn register(Json(body): Json<RegisterRequest>) {
    println!("{body:?}");
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login(Json(body): Json<LoginRequest>) {
    println!("{body:?}");
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
