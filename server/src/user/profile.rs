use axum::{
    Extension,
    extract::{Multipart, Path, State},
};
use axum_extra::{json, response::ErasedJson};
use common::AppError;
use database::{Db, user::User};
use std::sync::{Arc, Mutex};

pub async fn get_user_profile(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    Path(p): Path<String>,
) -> Result<ErasedJson, AppError> {
    let res = {
        let guard = user.lock().unwrap();
        if guard.username == p {
            Some(json!({
                "username": guard.username.clone(),
                "display_name": guard.display_name.clone(),
                "bio": guard.bio.clone(),
            }))
        } else {
            None
        }
    };

    if let Some(res) = res {
        Ok(res)
    } else {
        let u = db.get_user(&p).await?;
        Ok(json!({
            "username": u.username,
            "display_name": u.display_name,
            "bio": u.bio,
        }))
    }
}

pub async fn update_profile(
    State(db): State<Arc<Db>>,
    Extension(user): Extension<Arc<Mutex<User>>>,
    mut multipart: Multipart,
) -> Result<ErasedJson, AppError> {
    let (username, _id, mut icon, mut display_name, mut bio) = {
        let guard = user.lock().unwrap();
        (
            guard.username.clone(),
            guard._id.clone().to_string(),
            None,
            None,
            None,
        )
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
                filename = format!("icon/{_id}-{filename}");

                icon = Some(db.upload_image(&filename, data, &content_type).await?);
            }
            "display_name" => {
                let text = field.text().await.map_err(|e| {
                    tracing::error!("Invalid multipart/form-data field body: {e:?}");
                    AppError::InvalidData("Failed to read name")
                })?;

                common::validation::is_display_name_valid(&text)?;

                display_name = Some(text.trim().to_string());
            }
            "bio" => {
                let text = field.text().await.map_err(|e| {
                    tracing::error!("Invalid multipart/form-data field body: {e:?}");
                    AppError::InvalidData("Failed to read bio")
                })?;

                common::validation::is_bio_valid(&text)?;

                bio = Some(text);
            }
            _ => continue, // ignore unknown fields
        }
    }

    // update user profile in database
    db.update_profile(&username, &icon, &display_name, &bio)
        .await?;
    let res = {
        let mut guard = user.lock().unwrap();
        if icon.is_some() {
            guard.icon = icon;
        }
        if let Some(display_name) = display_name {
            guard.display_name = display_name;
        }
        if bio.is_some() {
            guard.bio = bio;
        }
        json!({
            "icon": guard.icon.clone(),
            "display_name": guard.display_name.clone(),
            "bio": guard.bio.clone(),
        })
    };

    Ok(res)
}
