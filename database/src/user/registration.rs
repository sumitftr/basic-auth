use crate::user::{User, UserStatus};
use common::{AppError, session::Session};
use mongodb::bson::{self, DateTime, doc};
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

        let applicant = User {
            legal_name: name.clone(),
            email: email.clone(),
            display_name: name,
            icon: Some(icon),
            status: Some(UserStatus::OidcVerified),
            ..Default::default()
        };

        match self.users.insert_one(&applicant).await {
            Ok(_) => {
                tracing::info!("[Created (OIDC)] Email: {}", applicant.email);
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
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

        let applicant = User {
            legal_name: name.clone(),
            email: email.clone(),
            birth_date: Some(birth_date),
            display_name: name,
            status: Some(UserStatus::Created(otp)),
            ..Default::default()
        };

        match self.users.insert_one(&applicant).await {
            Ok(_) => {
                tracing::info!("[Created] Email: {}", applicant.email);
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
        let status_bson = bson::to_bson(&UserStatus::Created(otp.to_string())).unwrap();
        let filter = doc! {"email": email, "status.variant": "Created" };
        let update = doc! {"$set": {"status": status_bson }};
        match self.users.update_one(filter, update).await {
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
        let filter = doc! {"email": email, "status.variant": "Created" };
        let applicant = match self.users.find_one(filter.clone()).await {
            Ok(Some(v)) => v,
            Ok(None) => return Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                return Err(AppError::ServerError);
            }
        };
        if let Some(UserStatus::Created(db_otp)) = applicant.status
            && db_otp == otp
        {
            let status_bson = bson::to_bson(&UserStatus::EmailVerified).unwrap();
            let update = doc! {"$set": {"status": status_bson }};
            match self.users.update_one(filter, update).await {
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
        let filter = doc! {"email": email, "status.variant": "EmailVerified"};
        let update = doc! {"$set": {"status.variant": "PasswordSet", "password": password}};
        match self.users.update_one(filter, update).await {
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
        new_session: Session,
    ) -> Result<User, AppError> {
        self.is_username_available(&username).await?;
        let filter =
            doc! {"email": &email, "status.variant": {"$in": ["PasswordSet", "OidcVerified"]}};
        let mut applicant = match self.users.find_one(filter.clone()).await {
            Ok(Some(v)) => v,
            Ok(None) => return Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                return Err(AppError::ServerError);
            }
        };

        // creating a new object in the bucket from the cdn url
        if applicant.icon.is_some() {
            let cdn_icon_url = applicant.icon.unwrap();
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
            applicant.icon = Some(
                self.upload_icon(data, filename, &applicant._id.to_string())
                    .await?,
            );
        }
        applicant.username = username;
        applicant.sessions = vec![new_session];
        applicant.created = DateTime::now();

        let sessions_bson = bson::to_bson(&applicant.sessions).unwrap();
        let update = doc! {"$set": {"username": &applicant.username, "sessions": sessions_bson, "icon": &applicant.icon, "created": &applicant.created }, "$unset": { "status": ""}};
        match self.users.update_one(filter, update).await {
            Ok(_) => {
                tracing::info!(
                    "[Registered] Email: {email}, Username: {}",
                    &applicant.username
                );
                Ok(applicant)
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }
}
