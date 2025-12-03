use super::{Applicant, ApplicationStatus};
use crate::user::User;
use common::AppError;
use mongodb::bson::{self, DateTime, doc, oid::ObjectId};
use std::sync::Arc;

// sub steps for registering an user
impl crate::Db {
    pub async fn create_applicant(
        self: Arc<Self>,
        name: String,
        email: String,
        birth_date: DateTime,
        otp: String,
    ) -> Result<(), AppError> {
        self.is_email_available(&email).await?;

        let applicant = Applicant {
            display_name: Some(name),
            email: email.clone(),
            birth_date: Some(birth_date),
            password: None,
            icon: None,
            phone: None,
            oauth_provider: None,
            status: ApplicationStatus::Created(otp),
        };

        match self.applicants.insert_one(&applicant).await {
            Ok(_) => {
                tracing::info!("[Created Applicant] Email: {}", applicant.email);
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    pub async fn update_applicant_otp(
        self: Arc<Self>,
        email: &str,
        otp: &str,
    ) -> Result<(), AppError> {
        let status_bson = bson::to_bson(&ApplicationStatus::Created(otp.to_string())).unwrap();
        let filter = doc! {"email": email, "status.tag": "Created"};
        // let filter = doc! {"email": email, "status": {"tag": "Created", "value": otp}};
        // let filter = doc! {"email": email, "status": {"tag": "Created", "value": {"$exists": true}}};
        let update = doc! {"$set": {"status": status_bson }};
        match self.applicants.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!("[OTP Changed] Email: {email}");
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    pub async fn verify_applicant_email(
        self: Arc<Self>,
        email: &str,
        otp: &str,
    ) -> Result<(), AppError> {
        let filter = doc! {"email": email, "status.tag": "Created"};
        // let filter = doc! {"email": email, "status.tag": "Created", "status.value": {"$exists": true}};
        // let filter = doc! {"email": email, "status": {"tag": "Created", "value": otp}};
        let applicant = match self.applicants.find_one(filter.clone()).await {
            Ok(Some(v)) => v,
            Ok(None) => return Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                return Err(AppError::ServerError);
            }
        };
        if let ApplicationStatus::Created(db_otp) = applicant.status
            && db_otp == otp
        {
            let status_bson = bson::to_bson(&ApplicationStatus::EmailVerified).unwrap();
            let update = doc! {"$set": {"status": status_bson }};
            match self.applicants.update_one(filter, update).await {
                Ok(_) => {
                    tracing::info!("[Email Verified] Email: {email}");
                    Ok(())
                }
                Err(e) => {
                    tracing::error!("{e:?}");
                    Err(AppError::ServerError)
                }
            }
        } else {
            Err(AppError::InvalidOTP)
        }
    }

    pub async fn set_applicant_password(
        self: Arc<Self>,
        email: &str,
        password: &str,
    ) -> Result<(), AppError> {
        let filter = doc! {"email": email, "status": {"tag": "EmailVerified"}};
        let update = doc! {"$set": {"password": password, "status.tag": "PasswordSet"}};
        match self.applicants.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!("[Password Set] Email: {email}, Password: {password}");
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    pub async fn set_applicant_username(
        self: Arc<Self>,
        email: String,
        username: String,
        new_session: common::session::Session,
    ) -> Result<User, AppError> {
        self.is_username_available(&username).await?;
        let filter = doc! {"email": &email, "status": {"tag": "PasswordSet"}};
        let applicant = match self.applicants.find_one_and_delete(filter).await {
            Ok(Some(v)) => v,
            Ok(None) => return Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                return Err(AppError::ServerError);
            }
        };

        let user = User {
            _id: ObjectId::new(),
            display_name: applicant.display_name.unwrap(),
            email,
            birth_date: applicant.birth_date.unwrap(),
            password: applicant.password,
            username,
            banner: None,
            icon: None,
            bio: None,
            legal_name: None,
            gender: None,
            phone: None,
            country: None,
            oauth_provider: None,
            sessions: vec![new_session],
            created: DateTime::now(),
        };
        self.create_user_forced(&user).await;
        Ok(user)
    }
}
