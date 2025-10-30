use common::{AppError, user_session::UserSession};
use mongodb::bson::{DateTime, doc};
use std::sync::Arc;

// implementation block for checking and updating user attributes
impl crate::Db {
    // updates email of the given user to new email
    pub async fn check_and_update_email(
        self: &Arc<Self>,
        email: &str,
        new_email: &str,
    ) -> Result<(), AppError> {
        self.is_email_available(email).await?;
        let update = doc! {"email": email};
        let filter = doc! {"$set": doc! {"email": new_email}};
        match self.users.update_one(filter, update).await {
            Ok(v) => {
                tracing::info!("Updated User Email: {:?}", v.upserted_id);
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerDefault)
            }
        }
    }

    // updates username of the given user to new username
    pub async fn check_and_update_username(
        self: &Arc<Self>,
        username: &str,
        new_username: &str,
    ) -> Result<(), AppError> {
        self.is_username_available(username).await?;
        let update = doc! {"username": username};
        let filter = doc! {"$set": doc! {"username": new_username}};
        match self.users.update_one(filter, update).await {
            Ok(v) => {
                tracing::info!("Updated Username: {:?}", v.upserted_id);
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerDefault)
            }
        }
    }

    // updates password of the given user
    pub async fn check_and_update_password(
        self: &Arc<Self>,
        username: &str,
        password: &str,
        new_password: &str,
    ) -> Result<(), AppError> {
        let update = doc! {"username": username};
        let filter = doc! {"$set": doc! {"password": new_password}};
        match self.users.find_one(update.clone()).await {
            Ok(Some(u)) => {
                if u.password != password {
                    Err(AppError::BadReq("Password didn't match"))
                } else {
                    match self.users.update_one(filter, update).await {
                        Ok(v) => {
                            tracing::info!("Updated User Password: {:?}", v.upserted_id);
                            Ok(())
                        }
                        Err(e) => {
                            tracing::error!("{e:?}");
                            Err(AppError::ServerDefault)
                        }
                    }
                }
            }
            Ok(None) => Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerDefault)
            }
        }
    }
}

// implementation block for just updating user attributes
impl crate::Db {
    // updates metadata for the given user
    pub async fn update_metadata(
        self: &Arc<Self>,
        username: &str,
        name: &str,
        gender: &str,
        dob: &DateTime,
    ) -> Result<(), AppError> {
        let update = doc! {"username": username};
        let filter = doc! {"$set": doc! {"name": name, "gender": gender, "dob": dob}};
        match self.users.update_one(filter, update).await {
            Ok(v) => {
                tracing::info!("Updated User Metadata: {:?}", v.upserted_id);
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerDefault)
            }
        }
    }

    pub async fn update_sessions(
        self: &Arc<Self>,
        username: &str,
        sessions: &Vec<UserSession>,
    ) -> Result<(), AppError> {
        todo!()
    }
}
