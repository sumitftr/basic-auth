use std::sync::Arc;
use util::AppError;

// implementation block for checking and updating user attributes by username
impl crate::Db {
    pub async fn check_and_update_username(
        self: &Arc<Self>,
        username: &str,
        new_username: &str,
    ) -> Result<(), AppError> {
        sqlx::query!("UPDATE users SET username = $1 WHERE username = $2", new_username, username)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("{:?}", e);
                AppError::ServerError
            })?;

        tracing::info!(
            "[Username Updated] Old Username: @{username}, New Username: @{new_username}"
        );
        Ok(())
    }

    pub async fn update_legal_name(
        self: &Arc<Self>,
        username: &str,
        legal_name: &str,
    ) -> Result<(), AppError> {
        sqlx::query!("UPDATE users SET legal_name = $1 WHERE username = $2", legal_name, username)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("{:?}", e);
                AppError::ServerError
            })?;

        tracing::info!("[Legal Name Updated] @{username}, Legal Name: {legal_name}");
        Ok(())
    }

    pub async fn update_birth_date(
        self: &Arc<Self>,
        username: &str,
        birth_date: sqlx::types::time::OffsetDateTime,
    ) -> Result<(), AppError> {
        sqlx::query!("UPDATE users SET birth_date = $1 WHERE username = $2", birth_date, username)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("{:?}", e);
                AppError::ServerError
            })?;

        tracing::info!("[Birth Date Updated] @{username}, Birth Date: {birth_date}");
        Ok(())
    }

    pub async fn update_gender(
        self: &Arc<Self>,
        username: &str,
        gender: &str,
    ) -> Result<(), AppError> {
        sqlx::query!("UPDATE users SET gender = $1 WHERE username = $2", gender, username)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("{:?}", e);
                AppError::ServerError
            })?;

        tracing::info!("[Gender Updated] @{username}, Gender: {gender}");
        Ok(())
    }

    pub async fn update_country(
        self: &Arc<Self>,
        username: &str,
        country: &str,
    ) -> Result<(), AppError> {
        sqlx::query!("UPDATE users SET country = $1 WHERE username = $2", country, username)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("{:?}", e);
                AppError::ServerError
            })?;

        tracing::info!("[Country Updated] @{username}, Country: {country}");
        Ok(())
    }

    // Update profile (dynamic fields)
    pub async fn update_profile(
        &self,
        username: &str,
        banner: &Option<String>,
        icon: &Option<String>,
        display_name: &Option<String>,
        bio: &Option<String>,
    ) -> Result<(), AppError> {
        // Build update query dynamically
        let mut updates = Vec::new();
        let mut params: Vec<String> = vec![username.to_string()];
        let mut param_index = 2;

        if let Some(val) = banner {
            updates.push(format!("banner = ${param_index}"));
            params.push(val.clone());
            param_index += 1;
        }
        if let Some(val) = icon {
            updates.push(format!("icon = ${param_index}"));
            params.push(val.clone());
            param_index += 1;
        }
        if let Some(val) = display_name {
            updates.push(format!("display_name = ${param_index}"));
            params.push(val.clone());
            param_index += 1;
        }
        if let Some(val) = bio {
            updates.push(format!("bio = ${param_index}"));
            params.push(val.clone());
        }

        if updates.is_empty() {
            return Ok(());
        }

        let query_str = format!("UPDATE users SET {} WHERE username = $1", updates.join(", "));

        let mut query = sqlx::query(&query_str);
        for param in params {
            query = query.bind(param);
        }

        query.execute(&self.pool).await.map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::ServerError
        })?;

        tracing::info!("[User Profile Updated] @{username}");
        Ok(())
    }
}
