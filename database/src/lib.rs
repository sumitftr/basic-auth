use moka::sync::Cache;
use mongodb::{Collection, error::ErrorKind};
use std::{sync::Arc, time::Duration};

pub mod active;
pub mod unregistered;
pub mod user;

pub static DATABASE_URI: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| std::env::var("DATABASE_URI").unwrap());

pub struct Db {
    users: Collection<crate::user::User>,
    // in memory stores
    active: Cache<common::user_session::ActiveUserSession, crate::user::User>,
    unregistered: Cache<String, crate::user::UnregisteredEntry>,
}

impl Db {
    pub async fn init() -> Arc<Self> {
        // establishing connection with local mongodb database
        let db = mongodb::Client::with_uri_str(&*DATABASE_URI)
            .await
            .unwrap()
            .database(&std::env::var("DATABASE_NAME").unwrap());

        // check and create all specified collections in `collections`
        let collections = ["users"];
        for collection in collections {
            if let Err(e) = db.create_collection(collection).await {
                match e.kind.as_ref() {
                    ErrorKind::Command(_) => {
                        tracing::error!("Collection `{}` already exists", collection);
                    }
                    _ => std::process::exit(1),
                }
            } else {
                tracing::info!("`{}` created", collection);
            }
        }

        Arc::new(Self {
            users: db.collection(collections[0]),
            active: Cache::builder()
                .max_capacity(32728)
                .time_to_live(Duration::from_secs(3600))
                .build(),
            unregistered: Cache::builder()
                .max_capacity(8192)
                .time_to_live(Duration::from_secs(1800))
                .build(),
        })
    }
}
