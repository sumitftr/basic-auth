use axum::{Json, extract::State};
use common::AppError;
use database::Db;
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct ForgotPasswordRequest {
    email: String,
}

pub async fn forgot_password(
    State(db): State<Arc<Db>>,
    Json(body): Json<ForgotPasswordRequest>,
) -> Result<(), AppError> {
    todo!()
}

#[derive(serde::Deserialize)]
pub struct ResetPasswordQuery {
    reset_password: String,
}

#[derive(serde::Deserialize)]
pub struct ResetPasswordRequest {
    password: String,
}

pub async fn reset_password(
    State(db): State<Arc<Db>>,
    Json(body): Json<ResetPasswordRequest>,
) -> Result<(), AppError> {
    todo!()
}
