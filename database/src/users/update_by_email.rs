use common::AppError;
use std::sync::Arc;

// implementation block for checking and updating user attributes by email
impl crate::Db {
    // updates password of the given user
    pub async fn update_password(
        self: &Arc<Self>,
        email: &str,
        password: &str,
    ) -> Result<(), AppError> {
        sqlx::query_as!(User, "UPDATE users SET password = $1 WHERE email = $2", password, email)
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
