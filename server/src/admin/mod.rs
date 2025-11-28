use axum::{Router, routing::get};
use std::sync::Arc;
use sysinfo::{Disks, Networks, System};
use tokio::sync::{Mutex, OnceCell};

mod health;

#[rustfmt::skip]
pub async fn admin_routes() -> Router {
    Router::new()
        .route("/api/health", get(health::health_handler))
        .layer(axum::middleware::from_fn(crate::middleware::admin_middleware))
        .layer(axum::middleware::from_fn(crate::middleware::auth_middleware))
        .with_state(database::Db::new().await)
}

#[derive(Clone)]
struct AdminState {
    sys: Arc<Mutex<System>>,
    networks: Arc<Mutex<Networks>>,
    disks: Arc<Mutex<Disks>>,
}

static ADMIN_STATE: OnceCell<AdminState> = OnceCell::const_new();

impl AdminState {
    pub async fn new() -> Self {
        ADMIN_STATE
            .get_or_init(|| async {
                AdminState {
                    sys: Arc::new(Mutex::new(System::new_all())),
                    networks: Arc::new(Mutex::new(Networks::new_with_refreshed_list())),
                    disks: Arc::new(Mutex::new(Disks::new_with_refreshed_list())),
                }
            })
            .await
            .clone()
    }
}
