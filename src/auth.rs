use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

pub fn auth_routes() -> Router {
    Router::new()
        .route("/", get(home_page))
        .route("/signup", get(signup_page).post(signup))
        .route("/login", get(login_page).post(login))
        .route("/logout", post(logout))
}

#[derive(Deserialize, Debug)]
pub struct LoginResponse {
    cookie: String,
    username: String,
    password: String,
}

pub async fn login(Json(body): Json<LoginResponse>) {
    println!("{body:?}");
    crate::auth::home_page().await
}

#[derive(Deserialize, Debug)]
enum Gender {
    Male,
    Female,
    Other,
}

#[derive(Deserialize, Debug)]
pub struct SignupResponse {
    email: String,
    username: String,
    password: String,
    name: String,
    gender: Gender,
    dob: (u8, u8, u16),
}

pub async fn signup(Json(body): Json<SignupResponse>) {
    println!("{body:?}");
    crate::auth::home_page().await
}

#[derive(Deserialize, Debug)]
pub struct LogoutResponse {
    username: String,
    cookie: String,
}

pub async fn logout(Json(body): Json<SignupResponse>) {
    println!("{body:?}");
    crate::auth::login_page().await
}

pub async fn home_page() {}
pub async fn login_page() {}
pub async fn signup_page() {}
