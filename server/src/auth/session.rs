use axum::{Json, extract::State};
use common::AppError;
use database::Db;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

pub async fn login(
    State(state): State<Arc<Db>>,
    Json(body): Json<LoginRequest>,
) -> Result<String, AppError> {
    todo!()
}

#[derive(Deserialize, Debug)]
pub struct LogoutRequest {
    // username: String,
}

pub async fn logout(Json(body): Json<LogoutRequest>) {
    println!("{body:?}");
}

pub async fn refresh() {}
