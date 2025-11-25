use crate::user::UserStatus;
use common::AppError;
use mongodb::bson::{self, doc};
use std::sync::Arc;

// implementation block for those users who forgot their password
impl crate::Db {
    pub async fn request_password_reset(
        self: &Arc<Self>,
        email: &str,
        code: String,
    ) -> Result<(), AppError> {
        let status_bson = bson::to_bson(&UserStatus::Recovering(code.clone())).unwrap();
        let filter = doc! {"email": email};
        let update = doc! {"$set": {"status": status_bson}};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!("[Password Reset Request] Email: {email}, Code: {code}");
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    // updates password of the given user (returns email)
    pub async fn reset_password(
        self: &Arc<Self>,
        code: &str,
        password: &str,
    ) -> Result<String, AppError> {
        let filter = doc! {"status.variant": "Recovering", "status.secret": code};
        let u = match self.users.find_one(filter.clone()).await {
            Ok(Some(u)) => u,
            Ok(None) => return Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                return Err(AppError::ServerError);
            }
        };
        let update = doc! {"$set": {"password": password}, "$unset": {"status": ""}};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!("[Password Reset] Email: {}, Password: {password}", &u.email);
                Ok(u.email)
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }
}
