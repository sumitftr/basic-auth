#![allow(dead_code, unused_variables)]

use axum::{Extension, Json, extract::State};
use database::{Db, UserData};
use std::sync::Arc;
use util::AppError;

#[derive(serde::Deserialize)]
pub struct UpdatePhoneRequest {
    phone: String,
}

pub async fn update_phone(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<UserData>,
    Json(body): Json<UpdatePhoneRequest>,
) -> Result<String, AppError> {
    todo!()
}

#[derive(serde::Deserialize)]
pub struct VerifyPhoneRequest {
    otp: String,
}

pub async fn verify_phone(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<UserData>,
    Json(body): Json<VerifyPhoneRequest>,
) -> Result<String, AppError> {
    todo!()
}
