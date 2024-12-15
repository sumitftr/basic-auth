use axum::Json;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct LogoutRequest {
    username: String,
}

pub async fn logout(Json(body): Json<LogoutRequest>) {
    println!("{body:?}");
}

pub async fn refresh_session() {}
