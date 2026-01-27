use axum::{extract::Request, middleware::Next, response::Response};
use database::users::User;
use util::AppError;

pub async fn admin_middleware(req: Request, next: Next) -> Result<Response, AppError> {
    if let Some(u) = req.extensions().get::<User>()
        && u.username == "admin"
    {
        Ok(next.run(req).await)
    } else {
        Err(AppError::NotFound)
    }
}
