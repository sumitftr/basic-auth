use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect},
};
use common::{AppError, session::ActiveSession};
use database::Db;
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct ProviderQuery {
    by: String,
}

pub async fn login(
    State(db): State<Arc<Db>>,
    Query(q): Query<ProviderQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Generate state, nonce, and PKCE
    let csrf_state = common::generate::random_string(32);
    let nonce = common::generate::random_string(32);
    let (code_verifier, code_challenge) = common::generate::pkce();
    let oauth_cfg =
        common::oauth::get_oauth_provider(common::oauth::OAuthProvider::try_from(q.by.as_str())?);

    db.add_oauth_creds(
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
        .append_pair("scope", "openid email profile")
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
    Query(q): Query<ProviderRedirect>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let (code_verifier, nonce, provider) = db
        .get_oauth_creds(&q.csrf_state)
        .ok_or_else(|| AppError::BadReq("CSRF State didn't match"))?;

    let oauth_cfg = common::oauth::get_oauth_provider(provider);
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
            ("code_verifier", &code_verifier),
        ])
        .send()
        .await
    {
        Ok(resp) => {
            if !resp.status().is_success() {
                tracing::error!(
                    "Token exchange failed: {}",
                    resp.text().await.unwrap_or_default()
                );
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

    let user_info = if let Some(id_token) = token_response.id_token {
        match verify_and_decode_id_token(&id_token, &oauth_cfg.client_id, &nonce) {
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

    // create applicant from oauth_oidc
    match db.get_user_by_email(&user_info.email).await {
        Ok(mut u) => match ActiveSession::parse_and_verify_from_headers(&headers) {
            Ok(active_session) => {
                db.make_user_active(active_session, u);
                db.remove_oauth_creds(&q.csrf_state);
                Ok(Redirect::to("/").into_response())
            }
            Err(_) => {
                let (db_session, active_session, set_cookie_headermap) =
                    common::session::create_session(&headers);
                u.sessions.push(db_session);
                db.update_sessions(&u.username, &u.sessions).await?;
                db.make_user_active(active_session, u);
                db.remove_oauth_creds(&q.csrf_state);
                Ok((set_cookie_headermap, Redirect::to("/")).into_response())
            }
        },
        Err(AppError::UserNotFound) => {
            db.create_applicant_oidc(user_info.name, user_info.email, user_info.picture)
                .await?;
            db.remove_oauth_creds(&q.csrf_state);
            Ok(Redirect::to("/register/set_username").into_response())
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
    // Decode without verification
    let token_data =
        jsonwebtoken::dangerous::insecure_decode::<IdTokenClaims>(token).map_err(|e| {
            tracing::error!("failed to decode id token: {e:?}");
            AppError::ServerError
        })?;

    let claims = token_data.claims;

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
