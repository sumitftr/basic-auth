mod banned_tokens;
mod users;

use mongodb::{error::ErrorKind, Collection};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

pub struct DBConf {
    users: Collection<crate::models::user::User>,
    banned_tokens: Mutex<HashSet<String>>,
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
            banned_tokens: Mutex::new(HashSet::new()),
        })
    }
}
