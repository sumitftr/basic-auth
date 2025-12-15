use crate::users::User;
use common::AppError;
use std::sync::Arc;

impl crate::Db {
    pub async fn delete_user_manual(self: &Arc<Self>, user: User) -> Result<(), AppError> {
        // Start a transaction for atomic operation
        let mut tx = self.pool.begin().await.map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::ServerError
        })?;

        // Insert into deleted_users
        sqlx::query!(
            r#"INSERT INTO deleted_users (
                id, display_name, email, birth_date, password, username, banner,
                icon, bio, legal_name, gender, phone, country, oauth_provider, created
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)"#,
            user.id,
            user.display_name,
            user.email,
            user.birth_date,
            user.password,
            user.username,
            user.banner,
            user.icon,
            user.bio,
            user.legal_name,
            user.gender,
            user.phone,
            user.country,
            user.oauth_provider.as_ref().map(|p| p.as_str()),
            user.created,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::ServerError
        })?;

        // Delete from users table
        sqlx::query!("DELETE FROM users WHERE id = $1", user.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("{:?}", e);
                AppError::ServerError
            })?;

        // Commit transaction
        tx.commit().await.map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::ServerError
        })?;

        tracing::info!("[User Deleted] Username: {}, Email: {}", user.username, user.email);
        Ok(())
    }
}
