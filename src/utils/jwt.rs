use crate::{database::DBConf, utils::AppError};
use axum::{
    extract::{ConnectInfo, State},
    http::{header, Request},
    response::Response,
};
use jsonwebtoken::errors::ErrorKind;
use std::sync::Arc;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // subject (username)
    pub iat: usize,  // issued at
    pub exp: usize,  // expiration time
    pub ip: String,  // client ip
}

// For more information on claims visit:
// https://www.iana.org/assignments/jwt/jwt.xhtml

pub fn generate(username: &str, ip: String) -> Result<String, AppError> {
    let now = chrono::Utc::now();
    let claims = Claims {
        sub: username.to_string(),
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::minutes(60)).timestamp() as usize,
        ip,
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(&*crate::SECRET_KEY.as_bytes()),
    )
    .map_err(|e| {
        let s = "Failed to create token";
        tracing::error!(s, username, "{e:?}");
        AppError::Server(s)
    })
}

// the middleware that validates requested user tokens
pub async fn authenticate(
    State(state): State<Arc<DBConf>>,
    ConnectInfo(conn_info): ConnectInfo<crate::utils::ClientConnInfo>,
    mut req: Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<Response, AppError> {
    // checking for the authorization header
    let Some(auth_header) = req.headers().get(header::AUTHORIZATION) else {
        return Err(AppError::BadReq("Authorization header not set"));
    };

    // stripping bearer from the authorization header
    let Ok(token) = auth_header
        .to_str()
        .map(|v| v.strip_prefix("Bearer ").unwrap())
    else {
        return Err(AppError::ServerDefault);
    };

    // checking if the token is banned
    if state.is_token_banned(token) {
        return Err(AppError::Auth("Token has expired"));
    }

    // decoding jwt token
    let tokendata = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(&*crate::SECRET_KEY.as_ref()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|e| match e.into_kind() {
        ErrorKind::InvalidToken => AppError::Auth("Token Invalid"),
        ErrorKind::InvalidSignature => AppError::Auth("Token has expired"),
        _ => AppError::Server("Authentication Failed"),
    })?;

    // checking ip
    if tokendata.claims.ip != conn_info.ip() {
        return Err(AppError::BadReq("Token Invalid"));
    }

    req.extensions_mut().insert(tokendata.claims);
    Ok(next.run(req).await)
}
