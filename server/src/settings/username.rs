use axum::{Extension, Json, extract::State};
use axum_extra::{json, response::ErasedJson};
use common::AppError;
use database::{Db, UserInfo};
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct UpdateUsernameRequest {
    new_username: String,
}

pub async fn update_username(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<UserInfo>,
    Json(body): Json<UpdateUsernameRequest>,
) -> Result<ErasedJson, AppError> {
    let username = user.lock().unwrap().0.username.clone();
    // checking whether the new username is same as original username or not
    if username == body.new_username {
        return Err(AppError::BadReq(
            "Your new username cannot be same as of your original username",
        ));
    }
    // checking if the new username is valid or not
    common::validation::is_username_valid(&body.new_username)?;
    // updating username in the primary database
    db.check_and_update_username(&username, &body.new_username).await?;
    user.lock().unwrap().0.username = body.new_username.clone();
    Ok(json!({
        "username": body.new_username,
        "message": "Your username has been updated"
    }))
}
