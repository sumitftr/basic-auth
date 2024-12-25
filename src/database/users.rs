use crate::{models::user::User, utils::AppError};
use mongodb::bson::{doc, DateTime};
use std::sync::Arc;

// implementation block for checking user attributes
impl super::DBConf {
    // checks if the email is available or not
    pub async fn is_email_available(self: &Arc<Self>, email: &str) -> Result<(), AppError> {
        match self.users.find_one(doc! { "email": email }).await {
            Ok(Some(_)) => Err(AppError::BadReq("Email not available")),
            Ok(None) => Ok(()),
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerDefault)
            }
        }
    }

    // checks if the username is available or not
    pub async fn is_username_available(self: &Arc<Self>, username: &str) -> Result<(), AppError> {
        match self.users.find_one(doc! { "username": username }).await {
            Ok(Some(_)) => Err(AppError::BadReq("Username not available")),
            Ok(None) => Ok(()),
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerDefault)
            }
        }
    }

    // matches database user's password with requested password
    pub async fn check_password(
        self: Arc<Self>,
        username: &str,
        password: &str,
    ) -> Result<(), AppError> {
        match self.users.find_one(doc! { "username": username }).await {
            Ok(Some(v)) if v.password == password => Ok(()),
            Ok(Some(_)) => Err(AppError::BadReq("Password didn't match")),
            Ok(None) => Err(AppError::UserNotFound),
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerDefault)
            }
        }
    }

    // adds a user to the database
    pub async fn add_user(self: Arc<Self>, user: &User) -> Result<(), AppError> {
        match self.users.insert_one(user).await {
            Ok(v) => {
                tracing::info!("Inserted User: {}", v.inserted_id);
                Ok(())
            }
            Err(e) => {
                tracing::error!("{e:?}");
                Err(AppError::ServerDefault)
            }
        }
    }
}

// implementation block for checking and updating user attributes
impl super::DBConf {
    // updates email of the given user to new email
    pub async fn check_and_update_email(
        self: Arc<Self>,
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
        self: Arc<Self>,
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
        self: Arc<Self>,
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
impl super::DBConf {
    // updates metadata for the given user
    pub async fn update_metadata(
        self: Arc<Self>,
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
}
