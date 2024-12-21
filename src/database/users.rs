use crate::models::user::User;
use mongodb::bson::{doc, DateTime};
use std::sync::Arc;

// implementation block for mutating users
impl super::DBConf {
    // checks if the email is available or not
    pub async fn is_email_available(self: &Arc<Self>, email: &str) -> mongodb::error::Result<()> {
        match self.users.find_one(doc! { "email": email }).await {
            Ok(v) => match v {
                Some(_) => Err(mongodb::error::Error::custom("Email not available")),
                None => Ok(()),
            },
            Err(e) => Err(e),
        }
    }

    // checks if the username is available or not
    pub async fn is_username_available(
        self: &Arc<Self>,
        username: &str,
    ) -> mongodb::error::Result<()> {
        match self.users.find_one(doc! { "username": username }).await {
            Ok(v) => match v {
                Some(_) => Err(mongodb::error::Error::custom("Username not available")),
                None => Ok(()),
            },
            Err(e) => Err(e),
        }
    }

    // adds a user to the database
    pub async fn check_and_add_user(self: Arc<Self>, user: &User) -> mongodb::error::Result<()> {
        self.is_email_available(&user.email).await?;
        self.is_username_available(&user.username).await?;
        match self.users.insert_one(user).await {
            Ok(v) => {
                tracing::info!("Inserted User: {}", v.inserted_id);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    // matches database user's password with requested password
    pub async fn check_password(
        self: Arc<Self>,
        username: &str,
        password: &str,
    ) -> mongodb::error::Result<()> {
        match self.users.find_one(doc! { "username": username }).await {
            Ok(Some(v)) => {
                if v.password == password {
                    Ok(())
                } else {
                    Err(mongodb::error::Error::custom("Password didn't match"))
                }
            }
            Ok(None) => Err(mongodb::error::Error::custom("Username not available")),
            Err(e) => Err(e),
        }
    }

    // updates email of the given user to new email
    pub async fn check_and_update_email(
        self: Arc<Self>,
        email: &str,
        new_email: &str,
    ) -> mongodb::error::Result<()> {
        self.is_email_available(email).await?;
        let update = doc! {"email": email};
        let filter = doc! {"$set": doc! {"email": new_email}};
        match self.users.update_one(filter, update).await {
            Ok(v) => {
                tracing::info!("Updated User Email: {:?}", v.upserted_id);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    // updates username of the given user to new username
    pub async fn check_and_update_username(
        self: Arc<Self>,
        username: &str,
        new_username: &str,
    ) -> mongodb::error::Result<()> {
        self.is_username_available(username).await?;
        let update = doc! {"username": username};
        let filter = doc! {"$set": doc! {"username": new_username}};
        match self.users.update_one(filter, update).await {
            Ok(v) => {
                tracing::info!("Updated Username: {:?}", v.upserted_id);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    // updates metadata for the given user
    pub async fn update_metadata(
        self: Arc<Self>,
        username: &str,
        name: &str,
        gender: &str,
        dob: &DateTime,
    ) -> mongodb::error::Result<()> {
        let update = doc! {"username": username};
        let filter = doc! {"$set": doc! {"name": name, "gender": gender, "dob": dob}};
        match self.users.update_one(filter, update).await {
            Ok(v) => {
                tracing::info!("Updated User Metadata: {:?}", v.upserted_id);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    // updates password of the given user
    pub async fn check_and_update_password(
        self: Arc<Self>,
        username: &str,
        password: &str,
        new_password: &str,
    ) -> mongodb::error::Result<()> {
        let update = doc! {"username": username};
        let filter = doc! {"$set": doc! {"password": new_password}};
        match self.users.find_one(update.clone()).await {
            Ok(Some(u)) => {
                if u.password != password {
                    return Err(mongodb::error::Error::custom("Password didn't match"));
                } else {
                    match self.users.update_one(filter, update).await {
                        Ok(v) => {
                            tracing::info!("Updated User Password: {:?}", v.upserted_id);
                            return Ok(());
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
            Ok(None) => return Err(mongodb::error::Error::custom("User not found")),
            Err(e) => return Err(e),
        };
    }
}
