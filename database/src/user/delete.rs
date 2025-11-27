use crate::user::User;
use common::AppError;
use mongodb::bson::doc;
use std::sync::Arc;

impl crate::Db {
    pub async fn delete_user(self: &Arc<Self>, user: User) -> Result<(), AppError> {
        let query = doc! { "username": &user.username };
        match self.users.delete_one(query).await {
            Ok(_) => {
                tracing::info!("[User Deleted] Username: {}, Email: {}", user.username, user.email);
                #[allow(clippy::unit_arg)]
                Ok(self
                    .deleted_users
                    .insert_one(&user)
                    .await
                    .map_or_else(|_| (), |e| tracing::error!("{e:?}"))) // have to return Ok(()) because the `User` is already deleted
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }
}
