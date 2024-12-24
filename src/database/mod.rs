use mongodb::{error::ErrorKind, Collection};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

mod banned_tokens;
mod unregistered;
mod users;

pub struct DBConf {
    users: Collection<crate::models::user::User>,
    unregistered: std::sync::Mutex<HashMap<String, crate::models::user::UnregisteredEntry>>,
    banned_tokens: std::sync::Mutex<HashSet<String>>,
}

impl DBConf {
    pub async fn init() -> Arc<Self> {
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

        Arc::new(Self {
            users: db.collection(collections[0]),
            unregistered: std::sync::Mutex::new(HashMap::new()),
            banned_tokens: std::sync::Mutex::new(HashSet::new()),
        })
    }
}
