use common::session::{ActiveSession, Session};
use moka::sync::Cache;
use mongodb::{Collection, error::ErrorKind};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::OnceCell;

pub mod applicant;
pub mod bucket;
pub mod mem;
pub mod user;

pub struct Db {
    users: Collection<user::User>,
    deleted_users: Collection<user::User>,
    applicants: Collection<applicant::Applicant>,
    bucket: bucket::BlackBlazeB2,
    // in memory stores
    active: Cache<ActiveSession, Arc<Mutex<user::User>>>,
    oauth_oidc: Cache<String, (String, String, common::oauth::OAuthProvider)>,
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
            let collections = ["users", "deleted_users", "applicants"];
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
                applicants: db.collection(collections[2]),
                bucket: bucket::BlackBlazeB2::default(),
                active: Cache::builder()
                    .max_capacity(32728)
                    .time_to_live(Duration::from_secs(Session::MEM_CACHE_DURATION))
                    .build(),
                oauth_oidc: Cache::builder()
                    .max_capacity(4096)
                    .time_to_live(Duration::from_secs(1800))
                    .build(),
            })
        })
        .await
        .clone()
    }
}
