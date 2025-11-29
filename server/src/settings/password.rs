use axum::{Extension, Json, extract::State};
use axum_extra::{json, response::ErasedJson};
use common::AppError;
use database::{Db, user::User};
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize)]
pub struct UpdatePasswordRequest {
    old_password: String,
    new_password: String,
}

pub async fn update_password(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    Json(body): Json<UpdatePasswordRequest>,
) -> Result<ErasedJson, AppError> {
    let email = {
        let guard = user.lock().unwrap();
        if guard.password.as_ref().unwrap() != &body.old_password {
            return Err(AppError::WrongPassword);
        }
        guard.email.clone()
    };
    common::validation::is_password_strong(&body.new_password)?;
    db.update_password(&email, &body.new_password).await?;
    user.lock().unwrap().password = Some(body.new_password);
    Ok(json!({
        "message": "Your password has been changed"
    }))
}
