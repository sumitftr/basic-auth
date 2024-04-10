use axum::{
    routing::{delete, get, patch},
    Router,
};

pub fn settings_routes() -> Router {
    Router::new()
        .route("/", get(crate::profile::profile))
        .route("/email", patch(change_email))
        .route("/username", patch(change_username))
        .route("/password", patch(reset_password))
        .route("/metadata", patch(change_metadata))
        .route("/account", delete(delete_account))
}

pub async fn change_email() {}
pub async fn change_username() {}
pub async fn reset_password() {}
pub async fn change_metadata() {}
pub async fn delete_account() {}
