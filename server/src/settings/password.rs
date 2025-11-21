use axum::{Extension, Json, extract::State};
use axum_extra::{json, response::ErasedJson};
use common::AppError;
use database::{Db, user::User};
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize)]
pub struct UpdatePasswordRequest {
    password: String,
}

pub async fn update_password(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    Json(body): Json<UpdatePasswordRequest>,
) -> Result<ErasedJson, AppError> {
    common::validation::is_password_valid(&body.password)?;
    let email = user.lock().unwrap().email.clone();
    db.update_password(&email, &body.password).await?;
    user.lock().unwrap().password = Some(body.password);
    Ok(json!({
        "message": "Your password has been changed"
    }))
}
