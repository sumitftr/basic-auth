use crate::user::User;
use common::AppError;
use std::sync::Arc;

impl crate::Db {
    // adds a user to the database
    pub async fn create_user(self: &Arc<Self>, user: &User) -> Result<(), AppError> {
        match self.users.insert_one(user).await {
            Ok(_) => {
                tracing::info!("[Registered] Username: {}, Email: {}", user.username, user.email);
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
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
