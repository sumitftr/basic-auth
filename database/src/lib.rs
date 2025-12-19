use common::session::Session;
use moka::sync::Cache;
use std::{sync::Arc, time::Duration};
use tokio::sync::OnceCell;

mod active;
pub mod applications;
pub mod bucket;
pub mod sessions;
pub mod users;

pub type UserData = Arc<std::sync::Mutex<(users::User, Vec<Session>)>>;

pub struct Db {
    pool: sqlx::Pool<sqlx::Postgres>,
    bucket: bucket::BlackBlazeB2,
    // in memory stores
    active: Cache<sqlx::types::Uuid, UserData>,
    applications: applications::Applications,
}

static DB: OnceCell<Arc<Db>> = OnceCell::const_new();

impl Db {
    pub async fn new() -> Arc<Self> {
        DB.get_or_init(|| async {
            // establishing connection with postgresql database
            let pool = sqlx::postgres::PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
                .await
                .unwrap();

            sqlx::migrate!("../.migrations").run(&pool).await.unwrap();

            Arc::new(Db {
                pool,
                bucket: bucket::BlackBlazeB2::default(),
                active: Cache::builder()
                    .max_capacity(32728)
                    .time_to_live(Duration::from_secs(Session::MEM_CACHE_DURATION))
                    .build(),
                applications: applications::Applications::new(),
            })
        })
        .await
        .clone()
    }

    pub fn logged_users_count(self: &Arc<Self>) -> u64 {
        self.active.entry_count()
    }
}
