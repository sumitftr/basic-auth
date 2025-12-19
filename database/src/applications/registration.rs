use super::{RegistrantEntry, RegistrantStatus};
use crate::users::User;
use common::AppError;
use sqlx::types::time::OffsetDateTime;
use std::{net::SocketAddr, sync::Arc};

// sub steps for registering an user
impl crate::Db {
    pub async fn create_registrant(
        self: &Arc<Self>,
        socket: SocketAddr,
        name: String,
        email: String,
        birth_date: OffsetDateTime,
        otp: String,
    ) -> Result<(), AppError> {
        self.is_email_available(&email).await?;
        self.applications.insert_registrant(
            email,
            RegistrantEntry {
                socket_addr: socket,
                display_name: Some(name),
                birth_date: Some(birth_date),
                password: None,
                icon: None,
                phone: None,
                oauth_provider: common::oauth::OAuthProvider::None,
                status: RegistrantStatus::Created(otp),
            },
        );
        Ok(())
    }

    pub async fn update_registrant_otp(
        self: &Arc<Self>,
        email: &str,
        otp: String,
    ) -> Result<(), AppError> {
        if let Some(mut entry) = self.applications.registrants.get(email) {
            entry.status = RegistrantStatus::Created(otp);
            self.applications.insert_registrant(email.to_string(), entry);
            Ok(())
        } else {
            Err(AppError::UserNotFound)
        }
    }

    pub async fn verify_registrant_email(
        self: &Arc<Self>,
        email: &str,
        otp: &str,
    ) -> Result<(), AppError> {
        let entry = self.applications.registrants.get(email).ok_or(AppError::UserNotFound)?;
        match &entry.status {
            RegistrantStatus::Created(db_otp) if db_otp == otp => {
                self.applications.insert_registrant(email.to_string(), entry);
                Ok(())
            }
            RegistrantStatus::Created(_) => Err(AppError::InvalidOTP),
            _ => Err(AppError::BadReq("Please verify the email")),
        }
    }

    pub async fn set_registrant_password(
        self: &Arc<Self>,
        email: &str,
        password: String,
    ) -> Result<(), AppError> {
        if let Some(mut entry) = self.applications.registrants.get(email) {
            entry.password = Some(password);
            self.applications.insert_registrant(email.to_string(), entry);
            Ok(())
        } else {
            Err(AppError::UserNotFound)
        }
    }

    pub async fn set_registrant_username(
        self: &Arc<Self>,
        email: String,
        username: String,
    ) -> Result<User, AppError> {
        self.is_username_available(&username).await?;
        let registrant = self.applications.registrants.get(&email).ok_or(AppError::UserNotFound)?;

        let user = User {
            id: sqlx::types::Uuid::new_v4(),
            display_name: registrant.display_name.unwrap(),
            email,
            birth_date: registrant.birth_date.unwrap(),
            password: registrant.password,
            username,
            banner: None,
            icon: None,
            bio: None,
            legal_name: None,
            gender: None,
            phone: None,
            country: None,
            oauth_provider: common::oauth::OAuthProvider::None,
            created: OffsetDateTime::now_utc(),
        };
        self.create_user_forced(&user).await;
        self.applications.remove_registrant(&user.email);
        Ok(user)
    }
}
