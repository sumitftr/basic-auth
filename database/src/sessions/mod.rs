use crate::users::User;
use common::AppError;
use common::session::ActiveSession;
use std::sync::Arc;

impl crate::Db {
    // this function finds the user by decrypted ssid (unsigned ssid)
    // but, this doesn't checks if the ssid is valid or not
    pub async fn get_user_by_active_session(
        self: &Arc<Self>,
        active_session: &ActiveSession,
    ) -> Result<User, AppError> {
        // Query for user where any session in the array matches the decrypted_ssid
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users 
            WHERE EXISTS (
                SELECT 1 FROM unnest(sessions) AS s 
                WHERE (s).unsigned_ssid = $1
            )
            "#,
            active_session.decrypted_ssid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::ServerError
        })?
        .ok_or(AppError::SessionExpired)?;

        Ok(user)
    }

    pub async fn update_sessions(
        self: &Arc<Self>,
        username: &str,
        sessions: &[Session],
    ) -> Result<(), AppError> {
        let sessions_json = serde_json::to_value(sessions).map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::ServerError
        })?;

        sqlx::query!("UPDATE users SET sessions = $1 WHERE username = $2", sessions_json, username)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("{:?}", e);
                AppError::ServerError
            })?;

        tracing::info!("[Session Updated]: @{username}");
        Ok(())
    }
}
