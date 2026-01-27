use crate::users::User;
use sqlx::types::{Uuid, ipnetwork::IpNetwork};
use std::sync::Arc;
use util::{
    AppError,
    session::{ParsedSession, Session},
};

impl crate::Db {
    /// returns the session that matches `parsed_session.unsigned_ssid`
    pub async fn get_session(
        self: &Arc<Self>,
        parsed_session: &ParsedSession,
    ) -> Result<Session, AppError> {
        let row = sqlx::query!(
            r#"SELECT * FROM sessions WHERE unsigned_ssid = $1 AND expires_at > NOW()"#,
            parsed_session.unsigned_ssid
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::ServerError
        })?
        .ok_or(AppError::SessionExpired)?;

        Ok(Session {
            unsigned_ssid: row.unsigned_ssid,
            user_agent: row.user_agent,
            ip_address: row.ip_address.ip(),
            created_at: row.created_at,
            last_used: row.last_used,
            expires_at: row.expires_at,
        })
    }

    /// returns the `User` and `Session` that matches the `parsed_session.unsigned_ssid`
    pub async fn get_all_by_parsed_session(
        self: &Arc<Self>,
        parsed_session: &ParsedSession,
    ) -> Result<(User, Session), AppError> {
        let row = sqlx::query!(
            r#"SELECT 
                u.id as user_id, u.display_name, u.email, u.birth_date, u.password, 
                u.username, u.banner, u.icon, u.bio, u.legal_name, u.gender, 
                u.phone, u.country, u.oauth_provider, u.created, s.unsigned_ssid,
                s.user_agent, s.ip_address, s.created_at, s.last_used, s.expires_at
            FROM users u
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

        let user = User {
            id: row.user_id,
            display_name: row.display_name,
            email: row.email,
            birth_date: row.birth_date,
            password: row.password,
            username: row.username,
            banner: row.banner,
            icon: row.icon,
            bio: row.bio,
            legal_name: row.legal_name,
            gender: row.gender,
            phone: row.phone,
            country: row.country,
            oauth_provider: util::oauth::OAuthProvider::from(row.oauth_provider.as_str()),
            created: row.created,
        };

        let session = Session {
            unsigned_ssid: row.unsigned_ssid,
            user_agent: row.user_agent,
            ip_address: row.ip_address.ip(),
            created_at: row.created_at,
            last_used: row.last_used,
            expires_at: row.expires_at,
        };

        Ok((user, session))
    }

    /// adds a session to sessions table
    pub async fn add_session(
        self: &Arc<Self>,
        user_id: Uuid,
        session: Session,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"INSERT INTO sessions (
                unsigned_ssid, user_id, user_agent, ip_address, created_at, last_used, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            session.unsigned_ssid,
            user_id,
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

        tracing::info!("[Session Added] user_id: {user_id}, session_id: {}", session.unsigned_ssid);
        Ok(())
    }

    /// removes the session that matches `unsigned_ssid`
    pub async fn remove_session(
        self: &Arc<Self>,
        user_id: Uuid,
        unsigned_ssid: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"DELETE FROM sessions WHERE unsigned_ssid = $1 AND user_id = $2"#,
            unsigned_ssid,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::ServerError
        })?;

        tracing::info!("[Session Removed] user_id: {}, session_id: {}", user_id, unsigned_ssid);
        Ok(())
    }

    /// removes all the session that matches `unsigned_ssids`
    pub async fn remove_selected_sessions(
        self: &Arc<Self>,
        user_id: Uuid,
        unsigned_ssids: &[Uuid],
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"DELETE FROM sessions WHERE user_id = $1 AND unsigned_ssid = ANY($2)"#,
            user_id,
            unsigned_ssids
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::ServerError
        })?;

        tracing::info!("[Sessions Removed] user_id: {}, count: {}", user_id, unsigned_ssids.len());
        Ok(())
    }

    /// removes all the session of User with `user_id` except the `except_unsigned_ssid`
    pub async fn remove_all_sessions(
        self: &Arc<Self>,
        user_id: Uuid,
        except_unsigned_ssid: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"DELETE FROM sessions WHERE user_id = $1 AND unsigned_ssid != $2"#,
            user_id,
            except_unsigned_ssid
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::ServerError
        })?;

        tracing::info!(
            "[All Sessions Removed] user_id: {}, except: {}",
            user_id,
            except_unsigned_ssid
        );
        Ok(())
    }

    /// removes all the expired sessions of User with `user_id`
    pub async fn clear_expired_sessions(self: &Arc<Self>, user_id: Uuid) -> Result<(), AppError> {
        sqlx::query!(r#"DELETE FROM sessions WHERE user_id = $1 AND expires_at <= NOW()"#, user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("{:?}", e);
                AppError::ServerError
            })?;

        tracing::info!("[Expired Sessions Cleared] user_id: {}", user_id);
        Ok(())
    }
}
