mod admin;
mod auth;
mod connection;
mod middleware;
mod settings;
mod stream_drop;
mod user;
mod user_data;

pub use connection::ClientSocket;

/// main router for server routes
pub async fn routes() -> axum::Router {
    axum::Router::new()
        .merge(admin::admin_routes().await)
        .merge(auth::auth_routes().await)
        .merge(settings::settings_routes().await)
        .merge(user::user_routes().await)
}

pub async fn get_custom_listener() -> stream_drop::CustomListener {
    use axum::serve::Listener;
    let listener = tokio::net::TcpListener::bind(std::env::var("SOCKET").unwrap()).await.unwrap();
    let custom_listener = stream_drop::CustomListener::from(listener);
    tracing::info!("[+] listening on {}", custom_listener.local_addr().unwrap());
    custom_listener
}
