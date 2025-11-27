use common::AppError;
use mongodb::bson::doc;
use std::sync::Arc;

// implementation block for checking and updating user attributes by email
impl crate::Db {
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
                tracing::info!("[Password Updated] Email: {email}, Password: {password}");
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }
}
