mod from_oidc;
mod recovery;
mod registration;
mod update;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Applicant {
    pub socket_addr: std::net::SocketAddr,
    pub display_name: Option<String>,
    pub email: String,
    pub birth_date: Option<mongodb::bson::DateTime>,
    pub password: Option<String>,
    pub icon: Option<String>,
    pub phone: Option<String>,
    pub oauth_provider: Option<common::oauth::OAuthProvider>,
    pub status: ApplicationStatus,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug, Clone)]
#[serde(tag = "tag", content = "value")]
pub enum ApplicationStatus {
    Created(String), // OTP
    EmailVerified,
    PasswordSet,
    OidcVerified,
    Recovering(String),                               // HEX HASH
    UpdatingEmail { old_email: String, otp: String }, // OTP
    UpdatingPhone { old_phone: String, otp: String }, // OTP
}

#[derive(Clone)]
pub struct OAuthInfo {
    pub csrf_state: String,
    pub code_verifier: String,
    pub nonce: String,
    pub provider: common::oauth::OAuthProvider,
}

impl crate::Db {
    pub async fn remove_applicant(
        self: std::sync::Arc<Self>,
        socket_addr: std::net::SocketAddr,
    ) -> Result<(), ()> {
        let socket_addr_bson = mongodb::bson::to_bson(&socket_addr).unwrap();
        let query = mongodb::bson::doc! { "socket_addr": socket_addr_bson };
        match self.applicants.delete_one(query).await {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }
}
