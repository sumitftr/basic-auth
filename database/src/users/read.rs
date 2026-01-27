use super::User;
use std::sync::Arc;
use util::AppError;

// implementation block for checking user attributes
impl crate::Db {
    // Check if email is available
    pub async fn is_email_available(&self, email: &str) -> Result<(), AppError> {
        if self.applications.is_email_present(email) {
            return Err(AppError::EmailTaken);
        }

        let exists =
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)", email)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    tracing::error!("{:?}", e);
                    AppError::ServerError
                })?;

        if exists.unwrap_or(false) { Err(AppError::EmailTaken) } else { Ok(()) }
    }

    // Check if username is available
    pub async fn is_username_available(&self, username: &str) -> Result<(), AppError> {
        let exists =
            sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)", username)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    tracing::error!("{:?}", e);
                    AppError::ServerError
                })?;

        if exists.unwrap_or(false) { Err(AppError::UsernameTaken) } else { Ok(()) }
    }

    // Authenticate user by email
    pub async fn authenticate_user_by_email(
        &self,
        email: &str,
        password: &str,
    ) -> Result<User, AppError> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("{:?}", e);
                AppError::ServerError
            })?
            .ok_or(AppError::UserNotFound)?;

        match &user.password {
            Some(db_password) if db_password == password => Ok(user),
            Some(_) => Err(AppError::PasswordMismatch),
            None => Err(AppError::BadReq("Password not set")),
        }
    }

    // Authenticate user by username
    pub async fn authenticate_user_by_username(
        &self,
        username: &str,
        password: &str,
    ) -> Result<User, AppError> {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("{:?}", e);
                AppError::ServerError
            })?
            .ok_or(AppError::UserNotFound)?;

        match &user.password {
            Some(db_password) if db_password == password => Ok(user),
            Some(_) => Err(AppError::PasswordMismatch),
            None => Err(AppError::BadReq("Password not set")),
        }
    }

    pub async fn get_user_by_email(self: &Arc<Self>, email: &str) -> Result<User, AppError> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::UserNotFound,
                _ => {
                    tracing::error!("{:?}", e);
                    AppError::ServerError
                }
            })
    }

    pub async fn get_user_by_username(self: &Arc<Self>, username: &str) -> Result<User, AppError> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE username = $1", username)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::UserNotFound,
                _ => {
                    tracing::error!("{:?}", e);
                    AppError::ServerError
                }
            })
    }
}
