use crate::{
    mem::{ApplicantEntry, ApplicantStatus},
    user::User,
};
use common::{AppError, session::Session};
use mongodb::bson::{DateTime, oid::ObjectId};
use std::sync::Arc;

// sub steps for registering an user
impl crate::Db {
    pub async fn create_applicant_oidc(
        self: &Arc<Self>,
        name: String,
        email: String,
        icon: String,
    ) -> Result<(), AppError> {
        // checking if the email is already used or not
        self.is_email_available(&email).await?;

        self.applicants.insert(
            email,
            ApplicantEntry {
                name,
                birth_date: None,
                otp: "".to_string(),
                password: None,
                icon: Some(icon),
                register_status: ApplicantStatus::OidcVerified,
            },
        );

        Ok(())
    }

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
                birth_date: Some(birth_date),
                otp,
                password: None,
                icon: None,
                register_status: ApplicantStatus::Created,
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
        new_session: Session,
    ) -> Result<User, AppError> {
        if let Some(mut entry) = self.applicants.get(&email) {
            if entry.register_status != ApplicantStatus::PasswordSet
                || entry.register_status != ApplicantStatus::OidcVerified
            {
                return Err(AppError::BadReq("User password not set"));
            }
            self.is_username_available(&username).await?;

            let _id = ObjectId::new();
            // creating a new object in the bucket from the cdn url
            if entry.icon.is_some() {
                let cdn_icon_url = entry.icon.unwrap();
                // Download the image from the source URL
                let response = reqwest::get(&cdn_icon_url).await.map_err(|e| {
                    tracing::error!("Failed to download image from {cdn_icon_url}: {e:#?}");
                    AppError::ServerError
                })?;

                if !response.status().is_success() {
                    tracing::error!(
                        "Failed to download image from {cdn_icon_url}: status {}",
                        response.status()
                    );
                    return Err(AppError::ServerError);
                }

                // Get the image data as bytes
                let data = response.bytes().await.map_err(|e| {
                    tracing::error!("Failed to read image bytes from {}: {e:#?}", cdn_icon_url);
                    AppError::ServerError
                })?;

                let filename = cdn_icon_url
                    .split('/')
                    .next_back()
                    .unwrap()
                    .split('=')
                    .next()
                    .unwrap()
                    .to_owned();
                entry.icon = Some(self.upload_icon(data, filename, &_id.to_string()).await?);
            }

            let user = User {
                _id,
                legal_name: entry.name.clone(),
                email: email.clone(),
                birth_date: entry.birth_date,
                password: entry.password,
                username,
                icon: entry.icon,
                display_name: entry.name,
                bio: None,
                gender: None,
                phone: None,
                country: None,
                sessions: vec![new_session],
                created: DateTime::now(),
                // last_login: DateTime::now(),
            };

            self.add_user(&user).await?;
            self.applicants.remove(&email).unwrap();

            Ok(user)
        } else {
            Err(AppError::UserNotFound)
        }
    }
}
