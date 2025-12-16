use super::{Applicant, ApplicationStatus};
use crate::users::User;
use common::AppError;
use sqlx::types::time::OffsetDateTime;
use std::{net::SocketAddr, sync::Arc};

// sub steps for registering an user
impl crate::Db {
    pub async fn create_applicant(
        self: &Arc<Self>,
        socket: SocketAddr,
        name: String,
        email: String,
        birth_date: OffsetDateTime,
        otp: String,
    ) -> Result<(), AppError> {
        self.is_email_available(&email).await?;
        self.applicants.insert(
            email,
            Applicant {
                socket_addr: socket,
                display_name: Some(name),
                birth_date: Some(birth_date),
                password: None,
                icon: None,
                phone: None,
                oauth_provider: None,
                status: ApplicationStatus::Created(otp),
            },
        );
        Ok(())
    }

    pub async fn update_applicant_otp(
        self: &Arc<Self>,
        email: &str,
        otp: String,
    ) -> Result<(), AppError> {
        if let Some(mut entry) = self.applicants.get(email) {
            entry.status = ApplicationStatus::Created(otp);
            self.applicants.insert(email.to_string(), entry);
            Ok(())
        } else {
            Err(AppError::UserNotFound)
        }
    }

    pub async fn verify_applicant_email(
        self: &Arc<Self>,
        email: &str,
        otp: &str,
    ) -> Result<(), AppError> {
        let entry = self.applicants.get(email).ok_or(AppError::UserNotFound)?;
        match &entry.status {
            ApplicationStatus::Created(db_otp) if db_otp == otp => {
                self.applicants.insert(email.to_string(), entry);
                Ok(())
            }
            ApplicationStatus::Created(_) => Err(AppError::InvalidOTP),
            _ => Err(AppError::BadReq("Please verify the email")),
        }
    }

    pub async fn set_applicant_password(
        self: &Arc<Self>,
        email: &str,
        password: String,
    ) -> Result<(), AppError> {
        if let Some(mut entry) = self.applicants.get(email) {
            entry.password = Some(password);
            self.applicants.insert(email.to_string(), entry);
            Ok(())
        } else {
            Err(AppError::UserNotFound)
        }
    }

    pub async fn set_applicant_username(
        self: &Arc<Self>,
        email: String,
        username: String,
        new_session: common::session::Session,
    ) -> Result<User, AppError> {
        self.is_username_available(&username).await?;
        let applicant = self.applicants.get(&email).ok_or(AppError::UserNotFound)?;

        let user = User {
            id: sqlx::types::Uuid::new_v4(),
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
            created: OffsetDateTime::now_utc(),
        };
        self.create_user_forced(&user).await;
        self.add_session(new_session).await?;
        self.applicants.remove(&user.email);
        Ok(user)
    }
}
