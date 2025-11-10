use crate::{
    mem::{ApplicantEntry, ApplicantStatus},
    user::User,
};
use common::{AppError, user_session::UserSession};
use mongodb::bson::{DateTime, oid::ObjectId};
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
        // checking if the email is already used or not
        self.is_email_available(&email).await?;

        self.applicants.insert(
            email,
            ApplicantEntry {
                name,
                birth_date,
                otp,
                password: None,
                register_status: ApplicantStatus::Created,
                session: vec![],
            },
        );

        Ok(())
    }

    pub fn update_applicant_otp(self: Arc<Self>, email: &str, otp: &str) -> Result<(), AppError> {
        if let Some(mut entry) = self.applicants.get(email) {
            if entry.register_status != ApplicantStatus::Created {
                return Err(AppError::BadReq("OTP can't be sent multiple times"));
            }
            entry.otp = otp.to_string();
            // inserting new entry to memory store
            self.applicants.insert(email.to_string(), entry);
            Ok(())
        } else {
            Err(AppError::UserNotFound)
        }
    }

    pub async fn verify_applicant_email(
        self: Arc<Self>,
        email: &str,
        otp: &str,
    ) -> Result<(), AppError> {
        if let Some(mut entry) = self.applicants.get(email) {
            if entry.register_status != ApplicantStatus::Created {
                return Err(AppError::BadReq("OTP can't be sent multiple times"));
            }
            if entry.otp != otp {
                return Err(AppError::BadReq("OTP doesn't match"));
            }
            entry.register_status = ApplicantStatus::EmailVerified;
            // inserting new entry to memory store
            self.applicants.insert(email.to_string(), entry);
            Ok(())
        } else {
            Err(AppError::UserNotFound)
        }
    }

    pub fn set_applicant_password(
        self: Arc<Self>,
        email: &str,
        password: String,
    ) -> Result<(), AppError> {
        if let Some(mut entry) = self.applicants.get(email) {
            if entry.register_status != ApplicantStatus::EmailVerified {
                return Err(AppError::BadReq("User email not verified"));
            }
            entry.password = Some(password);
            entry.register_status = ApplicantStatus::PasswordSet;
            self.applicants.insert(email.to_string(), entry);
            Ok(())
        } else {
            Err(AppError::UserNotFound)
        }
    }

    pub async fn set_applicant_username(
        self: Arc<Self>,
        email: String,
        username: String,
        new_session: UserSession,
    ) -> Result<User, AppError> {
        if let Some(entry) = self.applicants.get(&email) {
            if entry.register_status != ApplicantStatus::PasswordSet {
                return Err(AppError::BadReq("User password not set"));
            }
            self.is_username_available(&username).await?;
            let metadata = self.applicants.remove(&email).unwrap();

            let user = User {
                _id: ObjectId::new(),
                legal_name: metadata.name.clone(),
                email,
                birth_date: metadata.birth_date,
                password: metadata.password.unwrap(),
                username,
                profile_pic: None,
                display_name: metadata.name,
                bio: None,
                gender: None,
                phone: None,
                country: None,
                sessions: vec![new_session],
                created: DateTime::now(),
                // last_login: DateTime::now(),
            };
            self.add_user(&user).await?;

            Ok(user)
        } else {
            Err(AppError::UserNotFound)
        }
    }
}
