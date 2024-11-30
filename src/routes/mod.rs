mod auth;
mod profile;
mod settings;

use axum::{
    routing::{get, post},
    Router,
};

pub fn routes() -> Router {
    let app = Router::new()
        .route("/", get(home_page))
        .route("/register", get(auth::signup_page).post(auth::register))
        .route("/login", get(auth::login_page).post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/profile", get(profile::profile))
        .route("/profile", post(profile::update_profile))
        .nest("/settings", settings::settings_routes());

    return app;
}

pub async fn home_page() {}
