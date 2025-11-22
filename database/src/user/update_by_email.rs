use common::AppError;
use mongodb::bson::doc;
use std::sync::Arc;

// implementation block for checking and updating user attributes
impl crate::Db {
    // updates email of the given user to new email
    pub async fn update_email(
        self: &Arc<Self>,
        email: &str,
        new_email: &str,
    ) -> Result<(), AppError> {
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

    // updates password of the given user
    pub async fn update_password(
        self: &Arc<Self>,
        email: &str,
        password: &str,
    ) -> Result<(), AppError> {
        let filter = doc! {"email": email};
        let update = doc! {"$set": {"password": password}};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!("Email: {email}, New Password: {password}",);
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }
}
