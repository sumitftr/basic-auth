use crate::users::User;
use common::AppError;
use common::session::{ActiveSession, Session};
use sqlx::types::Uuid;
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

    pub async fn add_session(self: &Arc<Self>, session: Session) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"INSERT INTO sessions (
                unsigned_ssid, user_id, user_agent, ip_address, created_at, last_used, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            session.unsigned_ssid,
            session.user_id,
            session.user_agent,
            session.ip_address,
            session.created_at,
            session.last_used,
            session.expires_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::ServerError
        })?;

        tracing::info!("[Session Updated] id: {}", session.user_id.to_string());
        Ok(())
    }

    pub async fn remove_session(self: &Arc<Self>, user_id: Uuid) -> Result<(), AppError> {
        todo!()
    }

    pub async fn remove_selected_sessions(self: &Arc<Self>, user_id: Uuid) -> Result<(), AppError> {
        todo!()
    }

    pub async fn remove_all_sessions(self: &Arc<Self>, user_id: Uuid) -> Result<(), AppError> {
        todo!()
    }
}
