use common::{AppError, user_session::UserSession};
use mongodb::bson::doc;
use std::sync::Arc;

// implementation block for checking user attributes
impl crate::Db {
    // checks if the email is available or not
    pub async fn is_email_available(self: &Arc<Self>, email: &str) -> Result<(), AppError> {
        if self.unregistered.get(email).is_some() {
            return Err(AppError::EmailTaken);
        }
        match self.users.find_one(doc! { "email": email }).await {
            Ok(Some(_)) => Err(AppError::EmailTaken),
            Ok(None) => Ok(()),
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerDefault)
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
                Err(AppError::ServerDefault)
            }
        }
    }

    // matches database user's password with the specified password
    pub async fn check_password(
        self: &Arc<Self>,
        username: &str,
        password: &str,
    ) -> Result<(), AppError> {
        match self.users.find_one(doc! { "username": username }).await {
            Ok(Some(v)) if v.password == password => Ok(()),
            Ok(Some(_)) => Err(AppError::BadReq("Password didn't match")),
            Ok(None) => Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerDefault)
            }
        }
    }

    // this function finds the user by decrypted ssid (unsigned ssid)
    // but, this doesn't checks if the ssid is valid or not
    pub async fn get_user_by_decrypted_ssid(
        self: &Arc<Self>,
        decrypted_ssid: &str,
    ) -> Result<super::User, AppError> {
        match self
            .users
            .find(doc! { "sessions": [ { "unsigned_ssid": decrypted_ssid } ]})
            .await
        {
            Ok(v) => {}
            Err(e) => {}
        }
        todo!()
    }
}
