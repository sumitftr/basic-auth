use super::{RegistrantEntry, RegistrantStatus};
use crate::users::User;
use common::AppError;
use sqlx::types::time::OffsetDateTime;
use std::{net::SocketAddr, sync::Arc};

// sub steps for registering an user
impl crate::Db {
    pub async fn create_registrant_oidc(
        self: &Arc<Self>,
        socket_addr: SocketAddr,
        name: String,
        email: String,
        icon: String,
        oauth_provider: common::oauth::OAuthProvider,
    ) -> Result<(), AppError> {
        self.is_email_available(&email).await?;
        self.applications.insert_registrant(
            email,
            RegistrantEntry {
                socket_addr,
                display_name: Some(name),
                birth_date: None,
                password: None,
                icon: Some(icon),
                phone: None,
                oauth_provider,
                status: RegistrantStatus::OpenIDConnected,
            },
        );
        Ok(())
    }

    pub async fn finish_oidc_application(
        self: &Arc<Self>,
        email: String,
        birth_date: OffsetDateTime,
        username: String,
    ) -> Result<User, AppError> {
        self.is_username_available(&username).await?;
        let mut registrant =
            self.applications.registrants.get(&email).ok_or(AppError::UserNotFound)?;

        let id = sqlx::types::Uuid::new_v4();
        // creating a new object in the bucket from the cdn url
        if registrant.icon.is_some() {
            let cdn_icon_url = registrant.icon.unwrap();
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
            registrant.icon = Some(self.upload_icon(data, filename, &id.to_string()).await?);
        }

        let user = User {
            id,
            display_name: registrant.display_name.unwrap(),
            email,
            birth_date,
            password: None,
            username,
            banner: None,
            icon: registrant.icon,
            bio: None,
            legal_name: None,
            gender: None,
            phone: None,
            country: None,
            oauth_provider: registrant.oauth_provider,
            created: OffsetDateTime::now_utc(),
        };
        self.create_user_forced(&user).await;
        self.applications.remove_registrant(&user.email);
        Ok(user)
    }
}
