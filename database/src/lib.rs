use common::session::Session;
use moka::sync::Cache;
use std::{sync::Arc, time::Duration};
use tokio::sync::OnceCell;

mod active;
pub mod applicants;
pub mod bucket;
pub mod sessions;
pub mod users;

pub struct Db {
    pool: sqlx::Pool<sqlx::Postgres>,
    bucket: bucket::BlackBlazeB2,
    // in memory stores
    active: Cache<sqlx::types::Uuid, Arc<std::sync::Mutex<(users::User, Vec<Session>)>>>,
    applicants: applicants::ApplicantsCache,
    openid_connecting: Cache<std::net::SocketAddr, applicants::OAuthInfo>,
    recovering: Cache<String, applicants::PasswordResetInfo>, // code
}

static DB: OnceCell<Arc<Db>> = OnceCell::const_new();

impl Db {
    pub async fn new() -> Arc<Self> {
        DB.get_or_init(|| async {
            // establishing connection with postgresql database
            let pool = sqlx::postgres::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
                .await
                .unwrap();

            sqlx::migrate!("./migrations").run(&pool).await.unwrap();

            Arc::new(Db {
                pool,
                bucket: bucket::BlackBlazeB2::default(),
                active: Cache::builder()
                    .max_capacity(32728)
                    .time_to_live(Duration::from_secs(Session::MEM_CACHE_DURATION))
                    .build(),
                applicants: applicants::ApplicantsCache::new(),
                openid_connecting: Cache::builder()
                    .max_capacity(4096)
                    .time_to_live(Duration::from_secs(300))
                    .build(),
                recovering: Cache::builder()
                    .max_capacity(4096)
                    .time_to_live(Duration::from_secs(300))
                    .build(),
            })
        })
        .await
        .clone()
    }

    pub fn logged_users_count(self: &Arc<Self>) -> u64 {
        self.active.entry_count()
    }
}
