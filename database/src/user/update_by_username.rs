use common::{AppError, session::Session};
use mongodb::bson::{DateTime, doc};
use std::sync::Arc;

// implementation block for checking and updating user attributes
impl crate::Db {
    // updates username of the given user to new username
    pub async fn check_and_update_username(
        self: &Arc<Self>,
        username: &str,
        new_username: &str,
    ) -> Result<(), AppError> {
        // checking if the new username is available or not
        self.is_username_available(new_username).await?;
        let filter = doc! {"username": username};
        let update = doc! {"$set": {"username": new_username}};
        match self.users.update_one(filter, update).await {
            Ok(v) => {
                tracing::info!(
                    "[{:?}] Old Username: @{}, New Username: @{}",
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

    pub async fn update_sessions(
        self: &Arc<Self>,
        username: &str,
        sessions: &[Session],
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
                tracing::info!("Updated User Sessions: @{}", username);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to update sessions: {e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    pub async fn update_legal_name(
        self: &Arc<Self>,
        username: &str,
        legal_name: &str,
    ) -> Result<(), AppError> {
        let filter = doc! {"username": username};
        let update = doc! {"$set": {"legal_name": legal_name}};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!("Updated Legal Name: @{}", username);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to update legal name: {e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    pub async fn update_birth_date(
        self: &Arc<Self>,
        username: &str,
        birth_date: DateTime,
    ) -> Result<(), AppError> {
        let filter = doc! {"username": username};
        let update = doc! {"$set": {"birth_date": birth_date}};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!("Updated Birth Date: @{}", username);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to update birth date: {e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    pub async fn update_gender(
        self: &Arc<Self>,
        username: &str,
        gender: &str,
    ) -> Result<(), AppError> {
        let filter = doc! {"username": username};
        let update = doc! {"$set": {"gender": gender}};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!("Updated Gender: @{}", username);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to update gender: {e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    pub async fn update_country(
        self: &Arc<Self>,
        username: &str,
        country: &str,
    ) -> Result<(), AppError> {
        let filter = doc! {"username": username};
        let update = doc! {"$set": {"country": country}};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!("Updated Country: @{}", username);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to update country: {e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    // updates profile for the given user
    pub async fn update_profile(
        self: &Arc<Self>,
        username: &str,
        icon: &Option<String>,
        display_name: &Option<String>,
        bio: &Option<String>,
    ) -> Result<(), AppError> {
        // Build the update document dynamically
        let mut set_doc = mongodb::bson::Document::new();
        if let Some(icon_val) = icon {
            set_doc.insert("icon", icon_val);
        }
        if let Some(display_name_val) = display_name {
            set_doc.insert("display_name", display_name_val);
        }
        if let Some(bio_val) = bio {
            set_doc.insert("bio", bio_val);
        }
        // If nothing to update, return early
        if set_doc.is_empty() {
            return Ok(());
        }

        let filter = doc! {"username": username};
        let update = doc! {"$set": set_doc};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!("Updated User Profile: @{}", username);
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }
}
