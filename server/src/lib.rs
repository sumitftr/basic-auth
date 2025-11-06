use axum::{
    Router,
    routing::{get, post},
};

mod auth;
mod middleware;
mod user;

/// main router for server routes
#[rustfmt::skip]
pub async fn routes() -> Router {
    let main_router = Router::new()
        // user read routes
        .route("/api/user/@{id}", get(user::profile::get_user_profile))
        // user update routes
        .route("/api/user/email", post(user::settings::update_email))
        .route("/api/user/username", post(user::settings::update_username))
        .route("/api/user/password", post(user::settings::update_password))
        .route("/api/user/metadata", post(user::settings::update_metadata))
        .route("/api/user/deactivate", post(user::settings::deactivate_account))
        // layer and state
        .layer(axum::middleware::from_fn(middleware::auth_middleware))
        .with_state(database::Db::new().await);

    main_router.merge(auth::auth_routes().await)
}
