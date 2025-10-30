use axum::{Json, extract::State};
use common::AppError;
use database::Db;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct LoginRequest {
    email: Option<String>,
    username: Option<String>,
    password: String,
}

pub async fn login(
    State(state): State<Arc<Db>>,
    Json(body): Json<LoginRequest>,
) -> Result<String, AppError> {
    todo!()
}

pub async fn logout(State(state): State<Arc<Db>>) {
    todo!()
}
