use crate::database::DBConf;
use axum::{
    extract::{ConnectInfo, State},
    http::{header, Request, StatusCode},
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

pub fn generate(username: &str, ip: String) -> Result<String, String> {
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
        let result = format!("Failed to create token");
        tracing::error!(result, username, "{e:?}");
        result
    })
}

// the middleware that validates requested user tokens
pub async fn authenticate(
    State(state): State<Arc<DBConf>>,
    ConnectInfo(conn_info): ConnectInfo<crate::utils::ClientConnInfo>,
    mut req: Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<Response, (StatusCode, String)> {
    // checking for the authorization header
    let Some(auth_header) = req.headers().get(header::AUTHORIZATION) else {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Authorization header not set"),
        ));
    };

    // stripping bearer from the authorization header
    let token = auth_header
        .to_str()
        .map(|v| v.strip_prefix("Bearer ").unwrap())
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "".to_string()))?;

    // checking if the token is banned
    if state.is_token_banned(token) {
        return Err((StatusCode::BAD_REQUEST, format!("Token has expired")));
    }

    // decoding jwt token
    let tokendata = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(&*crate::SECRET_KEY.as_ref()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|e| match e.into_kind() {
        ErrorKind::InvalidToken => (StatusCode::BAD_REQUEST, format!("Token Invalid")),
        ErrorKind::InvalidSignature => (StatusCode::BAD_REQUEST, format!("Token has expired")),
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Authentication Failed"),
        ),
    })?;

    // checking ip
    if tokendata.claims.ip != conn_info.ip() {
        return Err((StatusCode::BAD_REQUEST, format!("Token Invalid")));
    }

    req.extensions_mut().insert(tokendata.claims);
    Ok(next.run(req).await)
}
