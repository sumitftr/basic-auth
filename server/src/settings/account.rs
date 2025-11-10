use axum::{Extension, extract::State};
use common::{AppError, user_session::ActiveUserSession};
use database::{Db, user::User};
use std::sync::{Arc, Mutex};

pub async fn delete_account(
    State(db): State<Arc<Db>>,
    Extension(active_user_session): Extension<ActiveUserSession>,
    Extension(user): Extension<Arc<Mutex<User>>>,
) -> Result<String, AppError> {
    db.remove_active_user(&active_user_session);
    let u = user.lock().unwrap().clone(); // this clone can be avoided
    db.delete_user(u).await?;
    Ok("Your account has been deleted".to_string())
}
