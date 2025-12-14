use moka::sync::Cache;
use std::{net::SocketAddr, time::Duration};

mod post_oidc;
mod pre_oidc;
mod recovering;
mod registration;
mod updating;

#[derive(Clone)]
pub struct OAuthInfo {
    pub csrf_state: String,
    pub code_verifier: String,
    pub nonce: String,
    pub provider: common::oauth::OAuthProvider,
}

#[derive(Clone)]
pub struct Applicant {
    pub socket_addr: std::net::SocketAddr,
    pub display_name: Option<String>,
    pub birth_date: Option<sqlx::types::time::PrimitiveDateTime>,
    pub password: Option<String>,
    pub icon: Option<String>,
    pub phone: Option<String>,
    pub oauth_provider: Option<common::oauth::OAuthProvider>,
    pub status: ApplicationStatus,
}

#[derive(PartialEq, Debug, Clone)]
#[serde(tag = "tag", content = "value")]
pub enum ApplicationStatus {
    Created(String), // OTP
    EmailVerified,
    PasswordSet,
    OidcVerified,
    UpdatingEmail { old_email: String, otp: String }, // OTP
    UpdatingPhone { old_phone: String, otp: String }, // OTP
}

#[derive(Clone)]
pub struct PasswordResetInfo {
    pub email: String,
    pub socket_addr: std::net::SocketAddr,
}

pub struct ApplicantsCache {
    entries: Cache<String, Applicant>,            // Email to Metadata
    mapping: Cache<std::net::SocketAddr, String>, // Socket Address to Email
}

impl ApplicantsCache {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            entries: Cache::builder()
                .max_capacity(4096)
                .time_to_live(Duration::from_secs(3600))
                .build(),
            mapping: Cache::builder()
                .max_capacity(4096)
                .time_to_live(Duration::from_secs(3600))
                .build(),
        }
    }

    pub fn insert(&self, email: String, metadata: Applicant) {
        self.mapping.insert(metadata.socket_addr, email.clone());
        self.entries.insert(email, metadata);
    }

    pub fn get(&self, email: &str) -> Option<Applicant> {
        self.entries.get(email)
    }

    pub fn remove(&self, email: &str) -> Option<Applicant> {
        let entry = self.entries.remove(email);
        if let Some(ref applicant) = entry {
            self.mapping.remove(&applicant.socket_addr);
        }
        entry
    }

    pub fn drop(&self, socket_addr: &SocketAddr) -> Option<Applicant> {
        if let Some(email) = self.mapping.remove(socket_addr) {
            self.entries.remove(&email)
        } else {
            None
        }
    }
}

impl crate::Db {
    #[inline]
    pub fn drop_applicant(self: &std::sync::Arc<Self>, socket_addr: &SocketAddr) {
        self.applicants.drop(socket_addr);
    }
}
