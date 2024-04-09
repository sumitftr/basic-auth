use axum::{routing::get, Router};
use tokio;

#[tokio::main]
async fn main() {
    let app = Router::new().route(
        "/",
        get(|| async { axum::response::Html("<h1>Hello, World!</h1>") }),
    );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
