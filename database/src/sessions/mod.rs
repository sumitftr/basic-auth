use crate::users::User;
use common::{
    AppError,
    session::{ParsedSession, Session},
};
use sqlx::types::{Uuid, ipnetwork::IpNetwork};
use std::sync::Arc;

impl crate::Db {
    pub async fn get_user_by_parsed_session(
        self: &Arc<Self>,
        parsed_session: &ParsedSession,
    ) -> Result<(User, Session), AppError> {
        let user = sqlx::query_as!(
            User,
            r#"SELECT u.* FROM users u
            INNER JOIN sessions s ON s.user_id = u.id
            WHERE s.unsigned_ssid = $1 AND s.expires_at > NOW()"#,
            parsed_session.unsigned_ssid
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

    pub async fn get_session(
        self: &Arc<Self>,
        parsed_session: &ParsedSession,
    ) -> Result<Session, AppError> {
        todo!()
    }

    pub async fn add_session(self: &Arc<Self>, session: Session) -> Result<(), AppError> {
        sqlx::query!(
            r#"INSERT INTO sessions (
                unsigned_ssid, user_id, user_agent, ip_address, created_at, last_used, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            session.unsigned_ssid,
            session.user_id,
            session.user_agent,
            IpNetwork::from(session.ip_address),
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

    pub async fn remove_session(
        self: &Arc<Self>,
        user_id: Uuid,
        session: Uuid,
    ) -> Result<(), AppError> {
        todo!()
    }

    pub async fn remove_selected_sessions(
        self: &Arc<Self>,
        user_id: Uuid,
        uids: &[Uuid],
    ) -> Result<(), AppError> {
        todo!()
    }

    pub async fn remove_all_sessions(
        self: &Arc<Self>,
        user_id: Uuid,
        except: Uuid,
    ) -> Result<(), AppError> {
        todo!()
    }

    pub async fn clear_expired_sessions(self: &Arc<Self>, user_id: Uuid) -> Result<(), AppError> {
        todo!()
    }
}
