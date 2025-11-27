use super::{Applicant, ApplicationStatus};
use crate::user::User;
use common::AppError;
use mongodb::bson::{DateTime, doc, oid::ObjectId};
use std::sync::Arc;

// sub steps for registering an user
impl crate::Db {
    pub async fn create_applicant_oidc(
        self: Arc<Self>,
        name: String,
        email: String,
        icon: String,
    ) -> Result<(), AppError> {
        // checking if the email is already used or not
        self.is_email_available(&email).await?;

        let applicant = Applicant {
            display_name: Some(name),
            email,
            birth_date: None,
            password: None,
            icon: Some(icon),
            phone: None,
            status: ApplicationStatus::OidcVerified,
        };

        match self.applicants.insert_one(&applicant).await {
            Ok(_) => {
                tracing::info!("[Created OIDC Applicant] Email: {}", applicant.email);
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    pub async fn finish_oidc_application(
        self: Arc<Self>,
        email: String,
        birth_date: DateTime,
        username: String,
        new_session: common::session::Session,
    ) -> Result<User, AppError> {
        self.is_username_available(&username).await?;
        let filter = doc! {"email": &email, "status": {"tag": "OidcVerified"}};
        let mut applicant = match self.applicants.find_one_and_delete(filter).await {
            Ok(Some(v)) => v,
            Ok(None) => return Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                return Err(AppError::ServerError);
            }
        };

        let _id = ObjectId::new();
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
            applicant.icon = Some(self.upload_icon(data, filename, &_id.to_string()).await?);
        }

        let user = User {
            _id,
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
            sessions: vec![new_session],
            created: DateTime::now(),
        };
        self.create_user_forced(&user).await;
        Ok(user)
    }
}
