use crate::ClientSocket;
use axum::{
    extract::{ConnectInfo, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect},
};
use base64::Engine;
use common::{AppError, oauth::OAuthProvider};
use database::Db;
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct ProviderQuery {
    by: String,
}

pub async fn login(
    State(db): State<Arc<Db>>,
    ConnectInfo(conn_info): ConnectInfo<ClientSocket>,
    Query(q): Query<ProviderQuery>,
) -> Result<Redirect, AppError> {
    // Generate state, nonce, and PKCE
    let csrf_state = common::generate::random_string(32);
    let nonce = common::generate::random_string(32);
    let (code_verifier, code_challenge) = common::generate::pkce();
    let oauth_cfg = common::oauth::get_oauth_provider(OAuthProvider::from(q.by))
        .ok_or(AppError::InvalidOAuthProvider)?;

    db.add_oidc_info(
        *conn_info,
        csrf_state.clone(),
        code_verifier,
        nonce.clone(),
        oauth_cfg.provider,
    );

    let redirect_uri = format!("{}/api/oauth2/callback", &*common::SERVICE_DOMAIN);
    let mut request_uri = oauth_cfg.authorization_endpoint.clone();
    request_uri
        .query_pairs_mut()
        .append_pair("client_id", &oauth_cfg.client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", oauth_cfg.provider.get_scopes())
        .append_pair("state", &csrf_state)
        .append_pair("nonce", &nonce)
        .append_pair("code_challenge", &code_challenge)
        .append_pair("code_challenge_method", "S256");

    Ok(Redirect::to(request_uri.as_str()))
}

// Query parameters for OAuth callback
#[derive(serde::Deserialize)]
pub struct ProviderRedirect {
    #[serde(rename = "code")]
    pub authorization_code: String,
    #[serde(rename = "state")]
    pub csrf_state: String,
}

// Token response from provider
#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    id_token: Option<String>,
    token_type: String,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
}

pub async fn callback(
    State(db): State<Arc<Db>>,
    ConnectInfo(conn_info): ConnectInfo<ClientSocket>,
    headers: HeaderMap,
    Query(q): Query<ProviderRedirect>,
) -> Result<impl IntoResponse, AppError> {
    let oidc_info =
        db.get_oidc_info(&q.csrf_state).ok_or(AppError::BadReq("CSRF state didn't match"))?;

    let oauth_cfg = common::oauth::get_oauth_provider(oidc_info.provider)
        .ok_or(AppError::InvalidOAuthProvider)?;
    let client = reqwest::Client::new();
    let redirect_uri = format!("{}/api/oauth2/callback", &*common::SERVICE_DOMAIN);

    // Exchange authorization code for tokens
    let token_response = match client
        .post(oauth_cfg.token_endpoint)
        .form(&[
            ("client_id", &oauth_cfg.client_id),
            ("client_secret", &oauth_cfg.client_secret),
            ("code", &q.authorization_code),
            ("redirect_uri", &redirect_uri),
            ("grant_type", &"authorization_code".to_string()),
            ("code_verifier", &oidc_info.code_verifier),
        ])
        .send()
        .await
    {
        Ok(resp) => {
            if !resp.status().is_success() {
                tracing::error!("Token exchange failed: {}", resp.text().await.unwrap_or_default());
                return Err(AppError::ServerError);
            }
            resp.json::<TokenResponse>().await.map_err(|e| {
                tracing::error!("Error parsing token response: {e:?}");
                AppError::ServerError
            })?
        }
        Err(e) => {
            tracing::error!("Error exchanging code: {e:?}");
            return Err(AppError::ServerError);
        }
    };

    // function for fetching user info if the information is not present inside id_token
    let fetch_user_info = || async {
        let resp = client
            .get(oauth_cfg.userinfo_endpoint)
            .bearer_auth(&token_response.access_token)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Error calling userinfo endpoint: {e:?}");
                AppError::ServerError
            })?;

        resp.json::<UserInfo>().await.map_err(|e| {
            tracing::error!("Error fetching user info: {e:?}");
            AppError::ServerError
        })
    };

    // getting user info by decoding id_token or by `fetch_user_info` function
    let user_info = if let Some(id_token) = token_response.id_token {
        match verify_and_decode_id_token(&id_token, &oauth_cfg.client_id, &oidc_info.nonce) {
            Ok(claims) => UserInfo {
                sub: claims.userinfo.sub,
                email: claims.userinfo.email,
                name: claims.userinfo.name,
                picture: claims.userinfo.picture,
            },
            Err(e) => {
                tracing::error!("Error verifying ID token: {e:?}");
                fetch_user_info().await?
            }
        }
    } else {
        fetch_user_info().await?
    };

    match db.get_user_by_email(&user_info.email).await {
        // if the user found inside database
        Ok(user) => match user.oauth_provider {
            // return error if the user have already registered with password
            OAuthProvider::None => Err(AppError::BadReq(
                "Your account with this email already exists. Please login to link your account.",
            )),
            // login if the user is already registered with OIDC
            _ => {
                let (new_session, parsed_session, set_cookie_headermap) =
                    common::session::create_session(user.id, &headers, *conn_info);
                db.add_session(user.id, new_session.clone()).await?;
                // activating session by adding it to `Db::active`
                if let Some((arc_wrapped, is_session_present)) = db.get_active_user(&parsed_session)
                    && !is_session_present
                {
                    let mut guard = arc_wrapped.lock().unwrap();
                    guard.1.push(new_session);
                } else {
                    db.make_user_active(user, new_session);
                }
                db.remove_oidc_info(&q.csrf_state);
                Ok((set_cookie_headermap, Redirect::to("/")).into_response()) // REDIRECT ENDPOINT NEEDS TO BE CHECKED
            }
        },
        // create registrant if the user is trying to register using open id connect
        Err(AppError::UserNotFound) => {
            db.create_registrant_oidc(
                *conn_info,
                user_info.name,
                user_info.email,
                user_info.picture,
                oidc_info.provider,
            )
            .await?;
            db.remove_oidc_info(&q.csrf_state);
            Ok(Redirect::to("/register/finish_oidc").into_response())
        }
        Err(e) => Err(e),
    }
}

// User information from OIDC
#[derive(serde::Deserialize)]
struct UserInfo {
    sub: String,
    name: String,
    // given_name: Option<String>,
    // family_name: Option<String>,
    picture: String,
    email: String,
    // email_verified: Option<bool>,
    // locale: Option<String>,
}

// JWT Claims
#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct IdTokenClaims {
    #[serde(flatten)]
    userinfo: UserInfo,
    iss: String,
    aud: String,
    iat: u64,
    exp: u64,
    nonce: Option<String>,
}

// Verify and decode ID token (simplified - not validating signature)
fn verify_and_decode_id_token(
    token: &str,
    expected_audience: &str,
    expected_nonce: &str,
) -> Result<IdTokenClaims, AppError> {
    // Split JWT into parts (header.payload.signature)
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        tracing::error!("Invalid JWT format");
        return Err(AppError::ServerError);
    }

    // Decode the payload (second part)
    let payload_bytes = base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(parts[1]).map_err(|e| {
        tracing::error!("failed to decode JWT payload: {e:?}");
        AppError::ServerError
    })?;

    // Deserialize the claims
    let claims: IdTokenClaims = serde_json::from_slice(&payload_bytes).map_err(|e| {
        tracing::error!("failed to deserialize id token claims: {e:?}");
        AppError::ServerError
    })?;

    // Verify audience
    if claims.aud != expected_audience {
        tracing::error!("Audience mismatch");
        return Err(AppError::ServerError);
    }

    // Verify nonce
    if expected_nonce != claims.nonce.as_ref().unwrap() {
        tracing::error!("Nonce mismatch");
        return Err(AppError::ServerError);
    }

    Ok(claims)
}
