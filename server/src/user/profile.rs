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

pub async fn update_profile(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    mut multipart: Multipart,
) -> Result<Json<UpdateProfileResponse>, AppError> {
    let username = user.lock().unwrap().username.clone();
    let mut res = UpdateProfileResponse {
        icon: None,
        display_name: None,
        bio: None,
    };

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Invalid multipart/form-data field: {e:?}");
        AppError::InvalidData("Failed to read multipart field")
    })? {
        let name = field
            .name()
            .ok_or_else(|| AppError::InvalidData("Field has no name"))?;

        match name {
            "icon" => {
                let mut filename = field
                    .file_name()
                    .ok_or_else(|| AppError::InvalidData("No filename provided"))?
                    .to_string();

                let data = field.bytes().await.map_err(|e| {
                    tracing::error!("Invalid multipart/form-data field body: {e:?}");
                    AppError::InvalidData("Failed to read image")
                })?;

                // checking if the user sent icon is valid or not
                let content_type = common::validation::is_icon_valid(&mut filename, &data)?;

                res.icon = Some(db.upload_image(&filename, data, &content_type).await?);
            }
            "display_name" => {
                let text = field.text().await.map_err(|e| {
                    tracing::error!("Invalid multipart/form-data field body: {e:?}");
                    AppError::InvalidData("Failed to read name")
                })?;

                if text.trim().is_empty() {
                    return Err(AppError::InvalidData("Name cannot be empty"));
                }

                if !text.trim().is_ascii() {
                    return Err(AppError::InvalidData(
                        "Name can only contain ascii characters",
                    ));
                }

                res.display_name = Some(text.trim().to_string());
            }
            "bio" => {
                let text = field.text().await.map_err(|e| {
                    tracing::error!("Invalid multipart/form-data field body: {e:?}");
                    AppError::InvalidData("Failed to read bio")
                })?;

                res.bio = Some(text);
            }
            _ => continue, // ignore unknown fields
        }
    }

    // update user profile in database
    db.update_profile(&username, &res.icon, &res.display_name, &res.bio)
        .await?;

    Ok(Json(res))
}

#[derive(serde::Serialize)]
pub struct UpdateProfileResponse {
    icon: Option<String>,
    display_name: Option<String>,
    bio: Option<String>,
}
