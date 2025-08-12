use crate::user::User;
use common::AppError;
use std::sync::Arc;

impl crate::Db {
    // adds a user to the database
    pub async fn add_user(self: Arc<Self>, user: &User) -> Result<(), AppError> {
        match self.users.insert_one(user).await {
            Ok(v) => {
                tracing::info!("Inserted User: {}", v.inserted_id);
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerDefault)
            }
        }
    }
}
