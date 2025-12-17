use axum::{extract::Request, middleware::Next, response::Response};
use common::AppError;
use database::users::User;

pub async fn admin_middleware(req: Request, next: Next) -> Result<Response, AppError> {
    if let Some(u) = req.extensions().get::<User>()
        && u.username == "admin"
    {
        Ok(next.run(req).await)
    } else {
        Err(AppError::NotFound)
    }
}
