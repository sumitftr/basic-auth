use common::AppError;
use mongodb::bson::doc;
use std::sync::Arc;

use crate::user::UserStatus;

// implementation block for checking and updating user attributes
impl crate::Db {
    pub async fn request_email_update(
        self: &Arc<Self>,
        old_email: &str,
        new_email: String,
        otp: String,
    ) -> Result<(), AppError> {
        let status_bson = mongodb::bson::to_bson(&UserStatus::UpdatingEmail {
            email: new_email.clone(),
            otp,
        })
        .unwrap();
        let filter = doc! {"email": old_email};
        let update = doc! {"$set": {"status": status_bson}};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!("[Email Update Request] Old: {old_email}, New: {new_email}");
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    // checks and updates email of the given user (returns updated email)
    pub async fn update_email(
        self: &Arc<Self>,
        old_email: &str,
        otp: &str,
    ) -> Result<String, AppError> {
        let filter = doc! {"email": old_email, "status.variant": "UpdatingEmail"};
        let u = match self.users.find_one(filter.clone()).await {
            Ok(Some(u)) => u,
            Ok(None) => return Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                return Err(AppError::ServerError);
            }
        };

        if let Some(UserStatus::UpdatingEmail {
            email: new_email,
            otp: db_otp,
        }) = u.status
        {
            // checking if the new email is available or not
            self.is_email_available(&new_email).await?;
            if otp == db_otp {
                let update = doc! {"$set": {"email": &new_email}, "$unset": {"status": ""}};
                match self.users.update_one(filter, update).await {
                    Ok(_) => {
                        tracing::info!("[Email Updated] Old: {old_email}, New: {new_email}");
                        Ok(new_email)
                    }
                    Err(e) => {
                        tracing::error!("{e:?}");
                        Err(AppError::ServerError)
                    }
                }
            } else {
                Err(AppError::InvalidOTP)
            }
        } else {
            Err(AppError::ServerError)
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
