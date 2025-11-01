use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// mod conn_info;

pub static SOCKET: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| std::env::var("SOCKET").unwrap());

#[tokio::main]
async fn main() {
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
        .await
        .unwrap();
}
