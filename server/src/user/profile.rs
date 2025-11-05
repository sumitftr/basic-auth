use axum::{
    Extension, Json,
    extract::{Path, State},
};
use common::AppError;
use database::{Db, user::User};
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Serialize)]
pub struct GetUserProfileResponse {
    username: String,
    display_name: String,
    bio: Option<String>,
}

pub async fn get_user_profile(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    Path(p): Path<String>,
) -> Result<Json<GetUserProfileResponse>, AppError> {
    let res = {
        let guard = user.lock().unwrap();
        if guard.username == p {
            Some(GetUserProfileResponse {
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
        Ok(Json(GetUserProfileResponse {
            username: u.username,
            display_name: u.display_name,
            bio: u.bio,
        }))
    }
}
