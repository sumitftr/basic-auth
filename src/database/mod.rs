use crate::models::User;
use mongodb::{
    bson::{doc, DateTime},
    Collection, Database,
};
use std::sync::Arc;

pub async fn db_init() -> Arc<Database> {
    // establishing connection with local mongodb database
    let db = mongodb::Client::with_uri_str(&*crate::DATABASE_URI)
        .await
        .unwrap()
        .database("web_db");

    // check and create all specified collections in `target_c`
    let target_c = ["users"];
    let current_c = db.list_collection_names().await.unwrap();
    if current_c.len() != 0 {
        let mut j = 0;
        for i in 0..current_c.len() {
            while current_c[i] != target_c[j] {
                db.create_collection(target_c[j]).await.unwrap();
                j += 1;
            }
            j += 1;
        }
    } else {
        for i in 0..target_c.len() {
            db.create_collection(target_c[i]).await.unwrap();
        }
    }

    Arc::new(db)
}

// checks if the email is available or not
pub async fn is_email_available(db: &Database, email: &str) -> mongodb::error::Result<bool> {
    let users: Collection<User> = db.collection("users");
    match users.find_one(doc! { "email": email }).await {
        Ok(v) => match v {
            Some(_) => return Ok(false),
            None => return Ok(true),
        },
        Err(e) => return Err(e),
    }
}

// checks if the username is available or not
pub async fn is_username_available(db: &Database, username: &str) -> mongodb::error::Result<bool> {
    let users: Collection<User> = db.collection("users");
    match users.find_one(doc! { "username": username }).await {
        Ok(v) => match v {
            Some(_) => return Ok(false),
            None => return Ok(true),
        },
        Err(e) => return Err(e),
    }
}

// adds a user to the database
pub async fn add_user(db: &Database, user: &User) -> mongodb::error::Result<()> {
    let users: Collection<User> = db.collection("users");
    match users.insert_one(user).await {
        Ok(v) => {
            println!("Inserted User: {}", v.inserted_id);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

// updates email of the given user to new email
pub async fn check_and_update_email(
    db: &Database,
    email: &str,
    new_email: &str,
) -> mongodb::error::Result<()> {
    if is_email_available(db, email).await? == false {
        return Err(mongodb::error::Error::custom("Email not available"));
    }
    let users: Collection<User> = db.collection("users");
    let update = doc! {"email": email};
    let filter = doc! {"$set": doc! {"email": new_email}};
    match users.update_one(filter, update).await {
        Ok(v) => {
            println!("Updated User: {:?}", v.upserted_id);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

// updates username of the given user to new username
pub async fn check_and_update_username(
    db: &Database,
    username: &str,
    new_username: &str,
) -> mongodb::error::Result<()> {
    if is_email_available(db, username).await? == false {
        return Err(mongodb::error::Error::custom("Username not available"));
    }
    let users: Collection<User> = db.collection("users");
    let update = doc! {"username": username};
    let filter = doc! {"$set": doc! {"username": new_username}};
    match users.update_one(filter, update).await {
        Ok(v) => {
            println!("Updated User: {:?}", v.upserted_id);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

// updates metadata for the given user
pub async fn update_metadata(
    db: &Database,
    username: &str,
    name: &str,
    gender: &str,
    dob: &DateTime,
) -> mongodb::error::Result<()> {
    let users: Collection<User> = db.collection("users");
    let update = doc! {"username": username};
    let filter = doc! {"$set": doc! {"name": name, "gender": gender, "dob": dob}};
    match users.update_one(filter, update).await {
        Ok(v) => {
            println!("Updated User: {:?}", v.upserted_id);
            Ok(())
        }
        Err(e) => Err(e),
    }
}
