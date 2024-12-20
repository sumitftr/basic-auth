use crate::models::User;
use mongodb::{
    bson::{doc, DateTime},
    error::ErrorKind,
    Collection,
};
use std::sync::{Arc, Mutex};

pub struct WebDB {
    users: Collection<User>,
    deleted_sessions: Mutex<Vec<String>>,
}

impl WebDB {
    pub async fn init() -> Arc<WebDB> {
        // establishing connection with local mongodb database
        let db = mongodb::Client::with_uri_str(&*crate::DATABASE_URI)
            .await
            .unwrap()
            .database("web_db");

        // check and create all specified collections in `collections`
        let collections = ["users"];
        for i in 0..collections.len() {
            if let Err(e) = db.create_collection(collections[i]).await {
                match e.kind.as_ref() {
                    ErrorKind::Command(_) => {
                        tracing::error!("Collection `{}` already exists", collections[i])
                    }
                    _ => std::process::exit(1),
                }
            } else {
                tracing::info!("`{}` created", collections[i]);
            }
        }

        Arc::new(WebDB {
            users: db.collection(collections[0]),
            deleted_sessions: Mutex::new(Vec::new()),
        })
    }

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
                println!("Inserted User: {}", v.inserted_id);
                Ok(())
            }
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
                println!("Updated User Email: {:?}", v.upserted_id);
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
                println!("Updated Username: {:?}", v.upserted_id);
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
                println!("Updated User Metadata: {:?}", v.upserted_id);
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
                            println!("Updated User Password: {:?}", v.upserted_id);
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
