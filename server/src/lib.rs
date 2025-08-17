use axum::{
    Router,
    routing::{get, post},
};

mod auth;
// mod jwt;
mod session;
mod user;

pub static SECRET_KEY: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| std::env::var("SECRET_KEY").unwrap());

/// main router for server routes
pub async fn routes() -> Router {
    let db = database::Db::init().await;
    let main_router = Router::new()
        // session handling routes
        .route("/api/user/logout", post(session::logout))
        .route("/api/session/refresh", post(session::refresh_session))
        // user read routes
        .route("/api/user/:id", get(user::get_user))
        // user update routes
        .route("/api/email/update", post(user::change_email))
        .route("/api/username/update", post(user::change_username))
        .route("/api/password/update", post(user::change_password))
        .route("/api/password/reset", post(user::reset_password))
        .route("/api/metadata/update", post(user::change_metadata))
        .route("/api/account/deactivate", post(user::deactivate_account))
        .with_state(std::sync::Arc::clone(&db));

    main_router.merge(auth::auth_routes(db))
    // .layer(tower_http::trace::TraceLayer::new_for_http())
}
