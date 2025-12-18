use axum::{Extension, Json, extract::State};
use axum_extra::{json, response::ErasedJson};
use common::AppError;
use database::{Db, UserInfo};
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct UpdatePasswordRequest {
    old_password: String,
    new_password: String,
}

pub async fn update_password(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<UserInfo>,
    Json(body): Json<UpdatePasswordRequest>,
) -> Result<ErasedJson, AppError> {
    let email = {
        let guard = user.lock().unwrap();
        if guard.0.password.as_ref().unwrap() != &body.old_password {
            return Err(AppError::WrongPassword);
        }
        guard.0.email.clone()
    };
    common::validation::is_password_strong(&body.new_password)?;
    db.update_password(&email, &body.new_password).await?;
    user.lock().unwrap().0.password = Some(body.new_password);
    Ok(json!({
        "message": "Your password has been changed"
    }))
}
