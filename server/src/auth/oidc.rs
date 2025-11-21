use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use common::AppError;
use database::Db;
use std::sync::Arc;

#[derive(serde::Deserialize)]
pub struct Provider {
    by: String,
}

pub async fn login(
    State(db): State<Arc<Db>>,
    Query(q): Query<Provider>,
) -> Result<impl IntoResponse, AppError> {
    // Generate state, nonce, and PKCE
    let csrf_state = common::generate::random_string(32);
    let nonce = common::generate::random_string(32);
    let (code_verifier, code_challenge) = common::generate::pkce();
    let oauth_cfg =
        common::oauth::get_oauth_provider(common::oauth::get_oauth_provider_by_str(&q.by)?);

    db.add_oauth_creds(
        csrf_state.clone(),
        code_verifier,
        nonce.clone(),
        oauth_cfg.provider,
    );

    let redirect_uri = format!("{}/oauth2/callback", &*common::SERVICE_DOMAIN); // this hasn't been decided yet
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
pub struct AuthRequest {
    pub code: String,
    pub state: String,
}

// Token response from provider
#[derive(serde::Deserialize)]
struct TokenResponse {
    access_token: String,
    id_token: Option<String>,
    token_type: String,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
}

// Google User information from OIDC
#[derive(serde::Deserialize)]
struct GoogleUserInfo {
    sub: String,
    name: String,
    // given_name: String,
    // family_name: String,
    picture: String,
    email: String,
    // email_verified: bool,
    // locale: String,
}

// JWT Claims
#[derive(serde::Deserialize)]
struct IdTokenClaims<T> {
    #[serde(flatten)]
    userinfo: T,
    iss: String,
    aud: String,
    exp: u64,
    iat: u64,
    nonce: Option<String>,
}

pub async fn callback(
    State(db): State<Arc<Db>>,
    Query(q): Query<AuthRequest>,
) -> Result<impl IntoResponse, AppError> {
    let (code_verifier, nonce, provider) = db
        .get_oauth_creds(&q.state)
        .ok_or_else(|| AppError::BadReq("CSRF State didn't match"))?;

    let oauth_cfg = common::oauth::get_oauth_provider(provider);

    // Exchange authorization code for tokens
    let redirect_uri = format!("{}/oauth2/callback", &*common::SERVICE_DOMAIN); // this hasn't been decided yet
    let client = reqwest::Client::new();
    let token_response = match client
        .post(oauth_cfg.token_endpoint)
        .form(&[
            ("client_id", &oauth_cfg.client_id),
            ("client_secret", &oauth_cfg.client_secret),
            ("code", &q.code),
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

    // Verify ID token if present
    let user_info = if let Some(id_token) = token_response.id_token {
        match verify_and_decode_id_token::<GoogleUserInfo>(&id_token, &oauth_cfg.client_id, &nonce)
        {
            Ok(claims) => GoogleUserInfo {
                sub: claims.userinfo.sub,
                email: claims.userinfo.email,
                name: claims.userinfo.name,
                picture: claims.userinfo.picture,
            },
            Err(e) => {
                tracing::error!("Error verifying ID token: {e:?}");
                return Err(AppError::ServerError);
            }
        }
    } else {
        // Fallback: fetch user info from userinfo endpoint
        match client
            .get(oauth_cfg.userinfo_endpoint)
            .bearer_auth(&token_response.access_token)
            .send()
            .await
        {
            Ok(resp) => resp.json::<GoogleUserInfo>().await.map_err(|e| {
                tracing::error!("Error fetching user info: {e:?}");
                AppError::ServerError
            })?,
            Err(e) => {
                tracing::error!("Error calling userinfo endpoint: {e:?}");
                return Err(AppError::ServerError);
            }
        }
    };

    // create applicant from oauth_oidc
    db.create_applicant_oidc(user_info.name, user_info.email, user_info.picture)
        .await?;
    db.remove_oauth_creds(&q.state);

    Ok(Redirect::to("/"))
}

// Verify and decode ID token (simplified - not validating signature)
fn verify_and_decode_id_token<T>(
    token: &str,
    expected_audience: &str,
    expected_nonce: &str,
) -> Result<IdTokenClaims<T>, AppError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    // Decode without verification
    let token_data =
        jsonwebtoken::dangerous::insecure_decode::<IdTokenClaims<T>>(token).map_err(|e| {
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
