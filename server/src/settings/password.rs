use axum::{Extension, Json, extract::State};
use axum_extra::{json, response::ErasedJson};
use database::{Db, UserData};
use std::sync::Arc;
use util::AppError;

#[derive(serde::Deserialize)]
pub struct UpdatePasswordRequest {
    old_password: String,
    new_password: String,
}

pub async fn update_password(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<UserData>,
    Json(body): Json<UpdatePasswordRequest>,
) -> Result<ErasedJson, AppError> {
    let email = {
        let guard = user.lock().unwrap();
        if guard.0.password.as_ref().is_none_or(|v| v != &body.old_password) {
            return Err(AppError::PasswordMismatch);
        }
        guard.0.email.clone()
    };
    util::validation::is_password_strong(&body.new_password)?;
    db.update_password(&email, &body.new_password).await?;
    user.lock().unwrap().0.password = Some(body.new_password);
    Ok(json!({
        "message": "Your password has been changed"
    }))
}

#[derive(serde::Deserialize)]
pub struct VerifyPasswordRequest {
    password: String,
}

pub async fn verify_password(
    Extension(user): Extension<UserData>,
    Json(body): Json<VerifyPasswordRequest>,
) -> Result<ErasedJson, AppError> {
    let guard = user.lock().unwrap();
    if guard.0.password.as_ref().is_none_or(|v| v != &body.password) {
        Err(AppError::PasswordMismatch)
    } else {
        Ok(json!({
            "success": "Password correct"
        }))
    }
}
