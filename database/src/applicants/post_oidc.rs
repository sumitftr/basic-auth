use super::{Applicant, ApplicationStatus};
use crate::users::User;
use common::AppError;
use sqlx::types::time::OffsetDateTime;
use std::{net::SocketAddr, sync::Arc};

// sub steps for registering an user
impl crate::Db {
    pub async fn create_applicant_oidc(
        self: &Arc<Self>,
        socket_addr: SocketAddr,
        name: String,
        email: String,
        icon: String,
        oauth_provider: common::oauth::OAuthProvider,
    ) -> Result<(), AppError> {
        self.is_email_available(&email).await?;
        self.applicants.insert(
            email,
            Applicant {
                socket_addr,
                display_name: Some(name),
                birth_date: None,
                password: None,
                icon: Some(icon),
                phone: None,
                oauth_provider,
                status: ApplicationStatus::OidcVerified,
            },
        );
        Ok(())
    }

    pub async fn finish_oidc_application(
        self: &Arc<Self>,
        email: String,
        birth_date: OffsetDateTime,
        username: String,
        new_session: common::session::Session,
    ) -> Result<User, AppError> {
        self.is_username_available(&username).await?;
        let mut applicant = self.applicants.get(&email).ok_or(AppError::UserNotFound)?;

        let id = sqlx::types::Uuid::new_v4();
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

            let filename =
                cdn_icon_url.split('/').next_back().unwrap().split('=').next().unwrap().to_owned();
            applicant.icon = Some(self.upload_icon(data, filename, &id.to_string()).await?);
        }

        let user = User {
            id,
            display_name: applicant.display_name.unwrap(),
            email,
            birth_date,
            password: None,
            username,
            banner: None,
            icon: applicant.icon,
            bio: None,
            legal_name: None,
            gender: None,
            phone: None,
            country: None,
            oauth_provider: applicant.oauth_provider,
            created: OffsetDateTime::now_utc(),
        };
        self.create_user_forced(&user).await;
        self.add_session(new_session).await?;
        self.applicants.remove(&user.email);
        Ok(user)
    }
}
