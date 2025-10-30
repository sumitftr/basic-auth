use axum::{
    Router,
    routing::{get, post},
};

mod auth;
mod middleware;
mod user;

/// main router for server routes
pub async fn routes() -> Router {
    let main_router = Router::new()
        // user read routes
        .route("/api/user/:id", get(user::get_user))
        // user update routes
        .route("/api/email/update", post(user::update_email))
        .route("/api/username/update", post(user::update_username))
        .route("/api/password/update", post(user::update_password))
        .route("/api/password/reset", post(user::reset_password))
        .route("/api/metadata/update", post(user::change_metadata))
        .route("/api/account/deactivate", post(user::deactivate_account))
        .with_state(database::Db::new().await)
        .layer(axum::middleware::from_fn(middleware::auth_middleware));

    main_router.merge(auth::auth_routes().await)
    // .layer(tower_http::trace::TraceLayer::new_for_http())
}
