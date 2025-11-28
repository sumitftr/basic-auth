mod oidc;
mod recovery;
mod registration;
mod update;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Applicant {
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
