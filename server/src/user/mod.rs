use axum::{
    Router,
    routing::{get, post},
};

mod profile;
mod recovery;
mod settings;

#[rustfmt::skip]
pub async fn user_routes() -> Router {
    Router::new()
        // profile routes
        .route("/api/user/@{id}", get(profile::get_user_profile))
        .route("/api/user/profile", post(profile::update_profile))
        // settings routes
        .route("/api/settings", get(settings::settings))
        .route("/api/settings/email", post(settings::update_email))
        .route("/api/settings/username", post(settings::update_username))
        .route("/api/settings/password", post(settings::update_password))
        .route("/api/settings/birth_date", post(settings::update_birth_date))
        .route("/api/settings/gender", post(settings::update_gender))
        .route("/api/settings/phone", post(settings::update_phone))
        .route("/api/settings/country", post(settings::update_country))
        .route("/api/settings/forgot_password", post(recovery::forgot_password))
        .route("/api/settings/reset_password", post(recovery::reset_password))
        .route("/api/settings/disable_account", post(recovery::disable_account))
        .route("/api/settings/delete_account", post(recovery::delete_account))
        // layer and state
        .layer(axum::middleware::from_fn(crate::middleware::auth_middleware))
        .with_state(database::Db::new().await)
}
