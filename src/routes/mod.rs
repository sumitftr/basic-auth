mod account;
mod auth;

use axum::{
    http::HeaderMap,
    routing::{get, post},
    Router,
};

pub async fn routes() -> Router {
    Router::new()
        .route("/", get(home_page))
        .route("/register", get(register_page))
        .route("/login", get(login_page))
        .route("/profile", get(profile))
        .route("/settings", get(settings))
        // api routes
        .route("/api/user/register", post(auth::register))
        .route("/api/user/login", post(auth::login))
        .route("/api/user/logout", post(auth::logout))
        .route("/api/update/email", post(account::change_email))
        .route("/api/update/username", post(account::change_username))
        .route("/api/update/password", post(account::reset_password))
        .route("/api/update/metadata", post(account::change_metadata))
        .route("/api/delete/account", post(account::delete_account))
        .with_state(crate::database::db_init().await)
}

pub async fn home_page(headers_map: HeaderMap) -> String {
    if crate::sessions::check_token(&headers_map) {
        "good".to_string()
    } else {
        "bad".to_string()
    }
}
pub async fn login_page() {}
pub async fn register_page() {}
pub async fn profile() {}
pub async fn settings() {}
