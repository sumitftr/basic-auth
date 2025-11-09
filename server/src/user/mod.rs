use axum::routing::{get, post};

mod profile;

#[rustfmt::skip]
pub async fn user_routes() -> axum::Router {
    axum::Router::new()
        .route("/api/user/@{id}", get(profile::get_user_profile))
        .route("/api/user/profile", post(profile::update_profile))
        .layer(axum::middleware::from_fn(crate::middleware::auth_middleware))
        .with_state(database::Db::new().await)
}
