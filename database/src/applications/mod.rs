use moka::sync::Cache;
use std::{net::SocketAddr, time::Duration};

mod post_oidc;
mod pre_oidc;
mod recovering;
mod registration;
mod updating;

pub struct Applications {
    socket_index: Cache<SocketAddr, DropType>,
    registrants: Cache<String, RegistrantEntry>, // Email [post_oidc, registration, updating]
    oidconnect: Cache<String, OidcInfo>,         // CSRF State [pre_oidc]
    passwd_reset: Cache<String, String>,         // Code/Email [recovering]
}

#[derive(Clone)]
pub enum DropType {
    Registrant(String),
}

#[derive(Clone)]
pub struct RegistrantEntry {
    pub socket_addr: std::net::SocketAddr,
    pub display_name: Option<String>,
    pub birth_date: Option<sqlx::types::time::OffsetDateTime>,
    pub password: Option<String>,
    pub icon: Option<String>,
    pub phone: Option<String>,
    pub oauth_provider: common::oauth::OAuthProvider,
    pub status: RegistrantStatus,
}

#[derive(PartialEq, Debug, Clone)]
pub enum RegistrantStatus {
    Created(String), // OTP
    EmailVerified,
    PasswordSet,
    OpenIDConnected,
    UpdatingEmail { old_email: String, otp: String },
    UpdatingPhone { old_phone: String, otp: String },
}

#[derive(Clone)]
pub struct OidcInfo {
    pub socket_addr: SocketAddr,
    pub code_verifier: String,
    pub nonce: String,
    pub provider: common::oauth::OAuthProvider,
}

impl Applications {
    #[allow(clippy::new_without_default)]
    pub(super) fn new() -> Self {
        Self {
            socket_index: Cache::builder()
                .max_capacity(4096)
                .time_to_live(Duration::from_secs(3600))
                .build(),
            registrants: Cache::builder()
                .max_capacity(4096)
                .time_to_live(Duration::from_secs(3600))
                .build(),
            oidconnect: Cache::builder()
                .max_capacity(4096)
                .time_to_live(Duration::from_secs(300))
                .build(),
            passwd_reset: Cache::builder()
                .max_capacity(4096)
                .time_to_live(Duration::from_secs(300))
                .build(),
        }
    }

    fn insert_registrant(&self, email: String, metadata: RegistrantEntry) {
        self.socket_index.insert(metadata.socket_addr, DropType::Registrant(email.clone()));
        self.registrants.insert(email, metadata);
    }

    fn remove_registrant(&self, email: &str) -> Option<RegistrantEntry> {
        let drop_type = self.registrants.remove(email);
        if let Some(entry) = drop_type.as_ref() {
            self.socket_index.remove(&entry.socket_addr);
        }
        drop_type
    }

    pub fn is_email_present(&self, email: &str) -> bool {
        self.registrants.contains_key(email)
    }
}

impl crate::Db {
    /// This method is implemented like this to be extensible on new feature additions
    #[inline]
    pub fn drop_application(self: &std::sync::Arc<Self>, socket_addr: &SocketAddr) {
        if let Some(entry) = self.applications.socket_index.remove(socket_addr) {
            match entry {
                DropType::Registrant(email) => {
                    self.applications.registrants.remove(&email);
                }
            }
        }
    }
}
