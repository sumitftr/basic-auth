mod active;
mod applicant;
mod recovery;
mod verification;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ApplicantEntry {
    pub name: String,
    // pub email: String,
    pub birth_date: mongodb::bson::DateTime,
    pub otp: String,
    pub password: Option<String>,
    pub register_status: ApplicantStatus,
    pub session: Vec<String>,
}

#[derive(serde::Deserialize, PartialEq, Debug, Clone)]
pub enum ApplicantStatus {
    Created,
    EmailVerified,
    PasswordSet,
}
