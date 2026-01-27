use crate::users::User;
use std::sync::Arc;
use util::AppError;

impl crate::Db {
    // adds a user to the database
    pub async fn create_user(self: &Arc<Self>, user: &User) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"INSERT INTO users (
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
            user.oauth_provider.get_str(),
            user.created
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                tracing::info!("[Registered] Username: {}, Email: {}", user.username, user.email);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to create user: {:?}", e);
                if let Some(db_err) = e.as_database_error()
                    && db_err.code() == Some(std::borrow::Cow::Borrowed("23505"))
                {
                    if db_err.message().contains("email") {
                        return Err(AppError::EmailTaken);
                    } else {
                        return Err(AppError::UsernameTaken);
                    }
                }
                Err(AppError::ServerError)
            }
        }
    }

    pub async fn create_user_forced(self: &Arc<Self>, user: &User) {
        let mut t = 0;
        loop {
            match self.create_user(user).await {
                Ok(_) => break,
                Err(_) => {
                    if t < 3 {
                        tokio::time::sleep(std::time::Duration::from_secs(t)).await;
                        t += 1;
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
