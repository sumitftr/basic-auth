use crate::applicant::{Applicant, ApplicationStatus};
use common::AppError;
use mongodb::bson::doc;
use std::sync::Arc;

// implementation block for checking and updating user attributes by email
impl crate::Db {
    pub async fn request_email_update(
        self: &Arc<Self>,
        old_email: String,
        new_email: &str,
        otp: &str,
    ) -> Result<(), AppError> {
        let applicant = doc! {
            "$set": mongodb::bson::to_bson(&Applicant {
                display_name: None,
                email: new_email.to_string(),
                birth_date: None,
                password: None,
                icon: None,
                phone: None,
                oauth_provider: None,
                status: ApplicationStatus::UpdatingEmail {
                    old_email: old_email.clone(),
                    otp: otp.to_string(),
                },
            }).unwrap()
        };
        let filter = doc! {"status": {"tag": "UpdatingEmail", "value": {"email": &old_email, "otp": {"$exists": true}}}};
        let options = mongodb::options::UpdateOptions::builder().upsert(true).build();
        match self.applicants.update_one(filter, applicant).with_options(options).await {
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
        let filter = doc! {"status": {"tag": "UpdatingEmail", "value": {"old_email": old_email, "otp": otp}}};
        // let filter = doc! {"status": {"tag": "UpdatingEmail", "value": {"old_email": old_email}}};
        // let filter = doc! {"status": {"tag": "UpdatingEmail", "value": {"old_email": old_email, "otp": {"$exists": true}}}};
        let applicant = match self.applicants.find_one(filter).await {
            Ok(Some(v)) => v,
            Ok(None) => return Err(AppError::InvalidOTP),
            Err(e) => {
                tracing::error!("{e:?}");
                return Err(AppError::ServerError);
            }
        };

        // no need to check for email availablity

        // checking and updating old email to new email
        let filter = doc! {"email": old_email};
        let update = doc! {"$set": {"email": &applicant.email}};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!("[Email Updated] Old: {old_email}, New: {}", applicant.email);
                Ok(applicant.email)
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }
}
