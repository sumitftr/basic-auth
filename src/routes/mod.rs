mod auth;
mod profile;
mod settings;

use axum::{
    http::HeaderMap,
    routing::{get, post},
    Router,
};

pub async fn routes() -> Router {
    Router::new()
        .route("/", get(home_page))
        .route("/register", post(auth::register).get(auth::register_page))
        .route("/login", post(auth::login).get(auth::login_page))
        .route("/logout", post(auth::logout))
        .route("/profile", get(profile::profile))
        .route("/settings", get(settings::settings))
        .route("/update/email", post(settings::change_email))
        .route("/update/username", post(settings::change_username))
        .route("/update/password", post(settings::reset_password))
        .route("/update/metadata", post(settings::change_metadata))
        .route("/delete/account", post(settings::delete_account))
        .with_state(crate::database::db_init().await)
}

pub async fn home_page(headers_map: HeaderMap) -> String {
    if crate::sessions::check_token(&headers_map) {
        "good".to_string()
    } else {
        "bad".to_string()
    }
}
