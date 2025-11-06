mod auth;
mod middleware;
mod user;

/// main router for server routes
pub async fn routes() -> axum::Router {
    let main_router = axum::Router::new();

    main_router
        .merge(auth::auth_routes().await)
        .merge(user::user_routes().await)
}
