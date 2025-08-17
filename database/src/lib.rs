use mongodb::{Collection, error::ErrorKind};
use std::{collections::HashMap, sync::Arc};

pub mod session;
pub mod user;

pub static DATABASE_URI: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| std::env::var("DATABASE_URI").unwrap());

pub struct Db {
    users: Collection<crate::user::User>,
    unregistered: std::sync::Mutex<HashMap<String, crate::user::UnregisteredEntry>>,
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
            unregistered: std::sync::Mutex::new(HashMap::new()),
        })
    }
}
