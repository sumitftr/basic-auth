use axum::{
    Json,
    http::{HeaderMap, StatusCode},
};

#[derive(Debug)]
pub enum AppError {
    BadReq(&'static str),
    InvalidData(&'static str),
    InvalidEmailFormat,
    UserNotFound,
    UsernameTaken,
    EmailTaken,
    WrongPassword,
    AuthError(&'static str),
    InvalidSession(HeaderMap),
    RefreshSession(HeaderMap),
    ServerError,
}

#[rustfmt::skip]
impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::BadReq(e) => {
                (StatusCode::BAD_REQUEST, JsonError::new(e)).into_response()
            }
            Self::InvalidData(e) => {
                (StatusCode::BAD_REQUEST, JsonError::new(e)).into_response()
            }
            Self::InvalidEmailFormat => {
                (StatusCode::BAD_REQUEST, JsonError::new("Invalid Email Format")).into_response()
            }
            Self::UserNotFound => {
                (StatusCode::NOT_FOUND, JsonError::new("User not found")).into_response()
            }
            Self::UsernameTaken => {
                (StatusCode::CONFLICT, JsonError::new("Username already taken")).into_response()
            }
            Self::EmailTaken => {
                (StatusCode::CONFLICT, JsonError::new("Email already taken")).into_response() 
            }
            Self::WrongPassword => {
                (StatusCode::UNAUTHORIZED, JsonError::new("Password didn't match")).into_response()
            }
            Self::AuthError(e) => {
                (StatusCode::UNAUTHORIZED, JsonError::new(e)).into_response()
            }
            Self::InvalidSession(set_cookies) => {
                (StatusCode::UNAUTHORIZED, set_cookies, JsonError::new("Invalid Session")).into_response()
            }
            Self::RefreshSession(set_cookies) => {
                (StatusCode::OK, set_cookies, JsonSuccess::new("Session Refreshed")).into_response()
            }
            Self::ServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, JsonError::new("Something went wrong")).into_response()
            }
        }
    }
}

#[derive(serde::Serialize)]
pub struct JsonError<'a> {
    error: &'a str,
}

impl<'a> JsonError<'a> {
    #[inline]
    pub fn new(error: &'a str) -> Json<Self> {
        Json(Self { error })
    }
}

#[derive(serde::Serialize)]
pub struct JsonSuccess<'a> {
    success: &'a str,
}

impl<'a> JsonSuccess<'a> {
    #[inline]
    pub fn new(success: &'a str) -> Json<Self> {
        Json(Self { success })
    }
}
