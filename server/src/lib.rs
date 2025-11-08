mod auth;
mod middleware;
mod settings;
mod user;

/// main router for server routes
pub async fn routes() -> axum::Router {
    axum::Router::new()
        .merge(auth::auth_routes().await)
        .merge(settings::settings_routes().await)
        .merge(user::user_routes().await)
}
