use crate::user::User;
use common::AppError;
use std::sync::Arc;

impl crate::Db {
    // adds a user to the database
    pub async fn add_user(self: &Arc<Self>, user: &User) -> Result<(), AppError> {
        match self.users.insert_one(user).await {
            Ok(_) => {
                tracing::info!(
                    "[Registered] Username: {}, Email: {}",
                    user.username,
                    user.email
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }
}
