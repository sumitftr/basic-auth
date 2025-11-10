use mongodb::bson::DateTime;
use serde::Deserialize;

mod register;

#[derive(Deserialize, Debug, Clone)]
pub struct ApplicantEntry {
    pub name: String,
    // pub email: String,
    pub birth_date: DateTime,
    pub otp: String,
    pub password: Option<String>,
    pub register_status: RegisterStatus,
    pub session: Vec<String>,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub enum RegisterStatus {
    Created,
    EmailVerified,
    PasswordSet,
}
