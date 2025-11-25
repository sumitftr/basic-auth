use super::User;
use common::{AppError, session::ActiveSession};
use mongodb::bson::doc;
use std::sync::Arc;

// implementation block for checking user attributes
impl crate::Db {
    // checks if the email is available or not
    pub async fn is_email_available(self: &Arc<Self>, email: &str) -> Result<(), AppError> {
        match self.users.find_one(doc! { "email": email }).await {
            Ok(Some(_)) => Err(AppError::EmailTaken),
            Ok(None) => Ok(()),
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    // checks if the username is available or not
    pub async fn is_username_available(self: &Arc<Self>, username: &str) -> Result<(), AppError> {
        match self.users.find_one(doc! { "username": username }).await {
            Ok(Some(_)) => Err(AppError::UsernameTaken),
            Ok(None) => Ok(()),
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    pub async fn authenticate_user_by_email(
        self: &Arc<Self>,
        email: &str,
        password: &str,
    ) -> Result<User, AppError> {
        match self.users.find_one(doc! { "email": email }).await {
            Ok(Some(user)) => {
                if let Some(db_password) = user.password.as_ref() {
                    if db_password == password {
                        Ok(user)
                    } else {
                        Err(AppError::WrongPassword)
                    }
                } else {
                    Err(AppError::BadReq("Password not set"))
                }
            }
            Ok(None) => Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    pub async fn authenticate_user_by_username(
        self: &Arc<Self>,
        username: &str,
        password: &str,
    ) -> Result<User, AppError> {
        match self.users.find_one(doc! { "username": username }).await {
            Ok(Some(user)) => {
                if let Some(db_password) = user.password.as_ref() {
                    if db_password == password {
                        Ok(user)
                    } else {
                        Err(AppError::WrongPassword)
                    }
                } else {
                    Err(AppError::BadReq("Password not set"))
                }
            }
            Ok(None) => Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    pub async fn get_user_by_email(self: &Arc<Self>, email: &str) -> Result<User, AppError> {
        match self.users.find_one(doc! { "email": email }).await {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    pub async fn get_user_by_username(self: &Arc<Self>, username: &str) -> Result<User, AppError> {
        if username.is_empty() {
            return Err(AppError::InvalidData("Username can't be empty"));
        }
        match self.users.find_one(doc! { "username": username }).await {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }

    // this function finds the user by decrypted ssid (unsigned ssid)
    // but, this doesn't checks if the ssid is valid or not
    pub async fn get_user_by_active_session(
        self: &Arc<Self>,
        active_session: &ActiveSession,
    ) -> Result<User, AppError> {
        let filter = doc! { "sessions": { "$elemMatch": { "unsigned_ssid": &active_session.decrypted_ssid } } };
        match self.users.find_one(filter).await {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(AppError::SessionExpired),
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerError)
            }
        }
    }
}
