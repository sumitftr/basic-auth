use axum::{Extension, extract::State};
use axum_extra::{json, response::ErasedJson};
use common::{AppError, session::ActiveSession};
use database::{Db, user::User};
use std::sync::{Arc, Mutex};

pub async fn delete_account(
    State(db): State<Arc<Db>>,
    Extension(active_session): Extension<ActiveSession>,
    Extension(user): Extension<Arc<Mutex<User>>>,
) -> Result<ErasedJson, AppError> {
    db.remove_active_user(&active_session);
    let u = user.lock().unwrap().clone(); // this clone can be avoided
    db.delete_user(u).await?;
    Ok(json!({
        "message": "Your account has been deleted"
    }))
}
