mod active;
mod applicant;
mod oauth_oidc;
mod recovery;
mod verification;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ApplicantEntry {
    pub name: String,
    // pub email: String,
    pub birth_date: Option<mongodb::bson::DateTime>,
    pub otp: String,
    pub password: Option<String>,
    pub icon: Option<String>,
    pub register_status: ApplicantStatus,
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub enum ApplicantStatus {
    Created,
    EmailVerified,
    PasswordSet,
    OidcVerified,
}
