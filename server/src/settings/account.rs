use axum::{Extension, extract::State};
use axum_extra::{json, response::ErasedJson};
use common::{AppError, session::ParsedSession};
use database::{Db, users::User};
use std::sync::{Arc, Mutex};

pub async fn delete_account(
    State(db): State<Arc<Db>>,
    Extension(parsed_session): Extension<ParsedSession>,
    Extension(user): Extension<Arc<Mutex<User>>>,
) -> Result<ErasedJson, AppError> {
    db.remove_active_user(&parsed_session);
    let u = user.lock().unwrap().clone(); // this clone can be avoided
    db.delete_user(u).await?;
    Ok(json!({
        "message": "Your account has been deleted"
    }))
}
