use axum::{
    routing::{get, post},
    Router,
};

pub fn settings_routes() -> Router {
    Router::new()
        .route("/", get(settings))
        .route("/email", post(change_email))
        .route("/username", post(change_username))
        .route("/password", post(reset_password))
        .route("/metadata", post(change_metadata))
        .route("/account/delete", post(delete_account))
}

pub async fn settings() {}
pub async fn change_email() {}
pub async fn change_username() {}
pub async fn reset_password() {}
pub async fn change_metadata() {}
pub async fn delete_account() {}
