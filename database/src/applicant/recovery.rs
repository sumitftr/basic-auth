use crate::applicant::{Applicant, ApplicationStatus};
use common::AppError;
use mongodb::bson::doc;
use std::sync::Arc;

// implementation block for those users who forgot their password
impl crate::Db {
    pub async fn request_password_reset(
        self: &Arc<Self>,
        email: &str,
        code: &str,
    ) -> Result<(), AppError> {
        let applicant = Applicant {
            display_name: None,
            email: email.to_string(),
            birth_date: None,
            password: None,
            icon: None,
            phone: None,
            status: ApplicationStatus::Recovering(code.to_string()),
        };
        match self.applicants.insert_one(applicant).await {
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
        let filter = doc! {"status": {"tag": "Recovering", "value": code}};
        let applicant = match self.applicants.find_one_and_delete(filter).await {
            Ok(Some(v)) => v,
            Ok(None) => return Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                return Err(AppError::ServerError);
            }
        };
        let filter = doc! {"email": &applicant.email};
        let update = doc! {"$set": {"password": password}};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!(
                    "[Password Reset] Email: {}, Password: {password}",
                    &applicant.email
                );
                Ok(applicant.email)
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }
}
