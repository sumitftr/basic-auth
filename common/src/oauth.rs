use reqwest::Url;
use std::sync::Arc;

use crate::AppError;

static OAUTH_PROVIDERS: std::sync::LazyLock<OAuthProviders> =
    std::sync::LazyLock::new(OAuthProviders::default);

struct OAuthProviders<'a> {
    /// https://accounts.google.com/.well-known/openid-configuration
    pub google: Arc<OAuthConfig<'a>>,
}

pub struct OAuthConfig<'a> {
    pub client_id: String,
    pub client_secret: String,
    pub authorization_endpoint: Url,
    pub token_endpoint: &'a str,
    pub userinfo_endpoint: &'a str,
    pub provider: OAuthProvider,
}

#[derive(Copy, Clone)]
pub enum OAuthProvider {
    Google,
}

impl<'a> Default for OAuthProviders<'a> {
    fn default() -> Self {
        Self {
            google: Arc::new(OAuthConfig {
                client_id: std::env::var("GOOGLE_CLIENT_ID").unwrap(),
                client_secret: std::env::var("GOOGLE_CLIENT_SECRET").unwrap(),
                authorization_endpoint: Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
                    .unwrap(),
                token_endpoint: "https://oauth2.googleapis.com/token",
                userinfo_endpoint: "https://openidconnect.googleapis.com/v1/userinfo",
                provider: OAuthProvider::Google,
            }),
        }
    }
}

pub fn get_oauth_provider_by_str(provider: &str) -> Result<OAuthProvider, AppError> {
    match provider {
        "google" => Ok(OAuthProvider::Google),
        _ => Err(AppError::BadReq("Invalid OAuth Provider")),
    }
}

pub fn get_oauth_provider(provider: OAuthProvider) -> Arc<OAuthConfig<'static>> {
    match provider {
        OAuthProvider::Google => OAUTH_PROVIDERS.google.clone(),
    }
}
