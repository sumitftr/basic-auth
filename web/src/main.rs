use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub static SOCKET: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| std::env::var("SOCKET").unwrap());

#[tokio::main]
pub async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing::level_filters::LevelFilter::from_level(
            tracing::Level::TRACE,
        ))
        .with(tracing_subscriber::fmt::Layer::default())
        .init();

    let router = server::routes().await;

    let listener = tokio::net::TcpListener::bind(&*SOCKET).await.unwrap();

    tracing::info!("[+] listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router) // app.into_make_service_with_connect_info::<conn_info::ClientConnInfo>(),
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

/// Shutdown signal to run axum with graceful shutdown when
/// a user presses Ctrl+C or Unix sends a terminate signal.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
