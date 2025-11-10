use axum::{Extension, Json, extract::State};
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
) -> Result<String, AppError> {
    common::validation::is_password_valid(&body.password)?;
    let email = user.lock().unwrap().email.clone();
    db.update_password(&email, &body.password).await?;
    Ok("Your password has been changed".to_string())
}
