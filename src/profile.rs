use axum::{routing::get, Router};

pub fn profile_routes() -> Router {
    Router::new().route("/profile", get(profile))
}

pub async fn profile() {}
