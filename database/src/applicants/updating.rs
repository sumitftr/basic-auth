use crate::applicants::{Applicant, ApplicationStatus};
use common::AppError;
use std::{net::SocketAddr, sync::Arc};

// implementation block for checking and updating user attributes by email
impl crate::Db {
    pub async fn request_email_update(
        self: &Arc<Self>,
        socket_addr: SocketAddr,
        old_email: String,
        new_email: String,
        otp: String,
    ) -> Result<(), AppError> {
        self.is_email_available(&new_email).await?;
        self.applicants.insert(
            new_email,
            Applicant {
                socket_addr,
                display_name: None,
                birth_date: None,
                password: None,
                icon: None,
                phone: None,
                oauth_provider: common::oauth::OAuthProvider::None,
                status: ApplicationStatus::UpdatingEmail { old_email, otp },
            },
        );
        Ok(())
    }

    // checks and updates email of the given user
    pub async fn update_email(
        self: &Arc<Self>,
        old_email: &str,
        new_email: String,
        otp: &str,
    ) -> Result<(), AppError> {
        let entry = self.applicants.get(&new_email).ok_or(AppError::UserNotFound)?;
        match &entry.status {
            ApplicationStatus::UpdatingEmail { old_email: mem_old_email, otp: mem_otp }
                if otp == mem_otp && old_email == mem_old_email =>
            {
                sqlx::query!("UPDATE users SET email = $1 WHERE email = $2", new_email, old_email)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| {
                        tracing::error!("{:?}", e);
                        AppError::ServerError
                    })?;

                tracing::info!("[Email Updated] Old: {old_email}, New: {new_email}");
                Ok(())
            }
            ApplicationStatus::UpdatingEmail { otp: mem_otp, .. } if otp != mem_otp => {
                Err(AppError::InvalidOTP)
            }
            ApplicationStatus::UpdatingEmail { .. } => {
                Err(AppError::BadReq("New email didn't match"))
            }
            _ => Err(AppError::BadReq("Please verify the email")),
        }
    }
}
