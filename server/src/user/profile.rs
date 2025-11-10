use axum::{
    Extension, Json,
    extract::{Multipart, Path, State},
};
use common::AppError;
use database::{Db, user::User};
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Serialize)]
pub struct UserProfileResponse {
    username: String,
    display_name: String,
    bio: Option<String>,
}

pub async fn get_user_profile(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    Path(p): Path<String>,
) -> Result<Json<UserProfileResponse>, AppError> {
    let res = {
        let guard = user.lock().unwrap();
        if guard.username == p {
            Some(UserProfileResponse {
                username: guard.username.clone(),
                display_name: guard.display_name.clone(),
                bio: guard.bio.clone(),
            })
        } else {
            None
        }
    };

    if let Some(res) = res {
        Ok(Json(res))
    } else {
        let u = db.get_user(&p).await?;
        Ok(Json(UserProfileResponse {
            username: u.username,
            display_name: u.display_name,
            bio: u.bio,
        }))
    }
}

#[allow(unused)]
pub async fn update_profile(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    mut multipart: Multipart,
) -> Result<Json<UserProfileResponse>, AppError> {
    if let Some(profile_picture) = multipart.next_field().await.unwrap() {}
    if let Some(display_name) = multipart.next_field().await.unwrap() {}
    if let Some(bio) = multipart.next_field().await.unwrap() {}
    todo!()
}
