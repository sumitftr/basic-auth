use axum::{Extension, Json, extract::State};
use common::AppError;
use database::{Db, user::User};
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize)]
pub struct UpdateUsernameRequest {
    new_username: String,
}

pub async fn update_username(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    Json(body): Json<UpdateUsernameRequest>,
) -> Result<String, AppError> {
    let username = user.lock().unwrap().username.clone();
    // checking whether the new username is same as original username or not
    if username == body.new_username {
        return Err(AppError::BadReq(
            "Your new username cannot be same as of your original username",
        ));
    }
    // checking if the new username is valid or not
    common::validation::is_username_valid(&body.new_username)?;
    // checking if the new username is available or not
    db.is_username_available(&body.new_username).await?;
    // updating username in the primary database
    db.update_username(&username, &body.new_username).await?;
    Ok(format!("Your new username is {}", body.new_username))
}
