use mongodb::{Collection, error::ErrorKind};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub mod session;
pub mod user;

pub static DATABASE_URI: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| std::env::var("DATABASE_URI").unwrap());

pub struct Db {
    users: Collection<crate::user::User>,
    unregistered: std::sync::Mutex<HashMap<String, crate::user::UnregisteredEntry>>,
    banned_tokens: std::sync::Mutex<HashSet<String>>,
}

impl Db {
    pub async fn init() -> Arc<Self> {
        // establishing connection with local mongodb database
        let db = mongodb::Client::with_uri_str(&*DATABASE_URI)
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
