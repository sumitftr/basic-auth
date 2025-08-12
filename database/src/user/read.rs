use common::AppError;
use mongodb::bson::doc;
use std::sync::Arc;

// implementation block for checking user attributes
impl crate::Db {
    // checks if the email is available or not
    pub async fn is_email_available(self: &Arc<Self>, email: &str) -> Result<(), AppError> {
        match self.users.find_one(doc! { "email": email }).await {
            Ok(Some(_)) => Err(AppError::BadReq("Email not available")),
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
            Ok(Some(_)) => Err(AppError::BadReq("Username not available")),
            Ok(None) => Ok(()),
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerDefault)
            }
        }
    }

    // matches database user's password with requested password
    pub async fn check_password(
        self: Arc<Self>,
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
}
