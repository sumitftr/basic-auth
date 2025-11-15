use crate::{mem::ApplicantEntry, user::User};
use common::user_session::{ActiveUserSession, UserSession};
use moka::sync::Cache;
use mongodb::{Collection, error::ErrorKind};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::OnceCell;

pub mod bucket;
pub mod mem;
pub mod user;

pub struct Db {
    users: Collection<User>,
    deleted_users: Collection<User>,
    bucket: bucket::BlackBlazeB2,
    // in memory stores
    active: Cache<ActiveUserSession, Arc<Mutex<User>>>,
    applicants: Cache<String, ApplicantEntry>,
    recovery: Cache<String, String>, // <QUERY_STRING, EMAIL>
    verification: Cache<String, (String, String)>, // <EMAIL|PHONE, (NEW_EMAIL|NEW_PHONE, OTP)>
}

static DB: OnceCell<Arc<Db>> = OnceCell::const_new();

impl Db {
    pub async fn new() -> Arc<Self> {
        DB.get_or_init(|| async {
            // establishing connection with local mongodb database
            let db = mongodb::Client::with_uri_str(std::env::var("DATABASE_URL").unwrap())
                .await
                .unwrap()
                .database(&std::env::var("DATABASE_NAME").unwrap());

            // check and create all specified collections in `collections`
            let collections = ["users", "deleted_users"];
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

            Arc::new(Db {
                users: db.collection(collections[0]),
                deleted_users: db.collection(collections[1]),
                bucket: bucket::BlackBlazeB2::default(),
                active: Cache::builder()
                    .max_capacity(32728)
                    .time_to_live(Duration::from_secs(UserSession::MEM_CACHE_DURATION))
                    .build(),
                applicants: Cache::builder()
                    .max_capacity(8192)
                    .time_to_live(Duration::from_secs(1800))
                    .build(),
                recovery: Cache::builder()
                    .max_capacity(8192)
                    .time_to_live(Duration::from_secs(1800))
                    .build(),
                verification: Cache::builder()
                    .max_capacity(4096)
                    .time_to_live(Duration::from_secs(1800))
                    .build(),
            })
        })
        .await
        .clone()
    }
}
