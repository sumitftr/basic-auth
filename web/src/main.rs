use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
// mod conn_info;
// mod jwt;

pub static SOCKET: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| std::env::var("SOCKET").unwrap());

#[tokio::main]
async fn main() {
    // let config_content = std::fs::read_to_string("../config.toml").unwrap();
    // let cfg: common::Config = toml::from_str(&config_content).unwrap();
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing::level_filters::LevelFilter::from_level(
            tracing::Level::TRACE,
        ))
        .with(tracing_subscriber::fmt::Layer::default())
        .init();

    let app = server::routes().await;

    let listener = tokio::net::TcpListener::bind(&*SOCKET).await.unwrap();

    tracing::info!("[+] listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app) // app.into_make_service_with_connect_info::<conn_info::ClientConnInfo>(),
        .await
        .unwrap();
}
