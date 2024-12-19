use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing::level_filters::LevelFilter::from_level(
            tracing::Level::TRACE,
        ))
        .with(tracing_subscriber::fmt::Layer::default())
        .init();

    let app = restfullapi::routes::routes().await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    tracing::info!("[+] listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<restfullapi::utils::ClientConnInfo>(),
    )
    .await
    .unwrap();
}
