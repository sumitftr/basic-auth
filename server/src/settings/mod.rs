use axum::routing::{get, post};

mod account;
mod email;
mod metadata;
mod password;
mod phone;
mod recovery;
mod username;

#[rustfmt::skip]
pub async fn settings_routes() -> axum::Router {
    axum::Router::new()
        .route("/api/settings", get(fetch_settings))
        .route("/api/settings/email", post(email::update_email))
        .route("/api/settings/verify_email", post(email::verify_email))
        .route("/api/settings/username", post(username::update_username))
        .route("/api/settings/password", post(password::update_password))
        .route("/api/settings/birth_date", post(metadata::update_birth_date))
        .route("/api/settings/gender", post(metadata::update_gender))
        .route("/api/settings/phone", post(phone::update_phone))
        .route("/api/settings/verify_phone", post(phone::verify_phone))
        .route("/api/settings/country", post(metadata::update_country))
        .route("/api/settings/forgot_password", post(recovery::forgot_password))
        .route("/api/settings/reset_password", post(recovery::reset_password))
        .route("/api/settings/delete_account", post(account::delete_account))
        // layer and state
        .layer(axum::middleware::from_fn(crate::middleware::auth_middleware))
        .with_state(database::Db::new().await)
}

pub async fn fetch_settings() {}
