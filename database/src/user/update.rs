use common::{AppError, user_session::UserSession};
use mongodb::bson::doc;
use std::sync::Arc;

// implementation block for checking and updating user attributes
impl crate::Db {
    // updates email of the given user to new email
    pub async fn check_and_update_email(
        self: &Arc<Self>,
        email: &str,
        new_email: &str,
    ) -> Result<(), AppError> {
        self.is_email_available(email).await?;
        let filter = doc! {"email": email};
        let update = doc! {"$set": {"email": new_email}};
        match self.users.update_one(filter, update).await {
            Ok(v) => {
                tracing::info!(
                    "[{:?}] Old Email: {}, New Email: {}",
                    v.upserted_id,
                    email,
                    new_email
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    // updates username of the given user to new username
    pub async fn check_and_update_username(
        self: &Arc<Self>,
        username: &str,
        new_username: &str,
    ) -> Result<(), AppError> {
        self.is_username_available(username).await?;
        let filter = doc! {"username": username};
        let update = doc! {"$set": {"username": new_username}};
        match self.users.update_one(filter, update).await {
            Ok(v) => {
                tracing::info!(
                    "[{:?}] Old Username: {}, New Username: {}",
                    v.upserted_id,
                    username,
                    new_username
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    // updates password of the given user
    pub async fn update_password(
        self: &Arc<Self>,
        username: &str,
        password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        let filter = doc! {"username": username};
        let update = doc! {"$set": {"password": new_password}};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!(
                    "Username: {username}, Old Password: {password}, New Password: {new_password}",
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    pub async fn update_sessions(
        self: &Arc<Self>,
        username: &str,
        sessions: &[UserSession],
    ) -> Result<(), AppError> {
        // Serialize the sessions array slice to Bson
        let sessions_bson = mongodb::bson::to_bson(sessions).map_err(|e| {
            tracing::error!("Failed to serialize sessions: {e}");
            AppError::ServerError
        })?;

        let filter = doc! {"username": username};
        let update = doc! {"$set": {"sessions": sessions_bson}};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!("Updated User Sessions: {}", username);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to update sessions: {e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    // updates profile for the given user
    pub async fn update_profile(
        self: &Arc<Self>,
        username: &str,
        profile_pic: &str,
        display_name: &str,
        bio: &str,
    ) -> Result<(), AppError> {
        let filter = doc! {"username": username};
        let update =
            doc! {"$set": {"profile_pic": profile_pic, "display_name": display_name, "bio": bio}};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!("Updated User Profile: {}", username);
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }
}
