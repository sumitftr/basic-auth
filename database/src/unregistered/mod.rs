use mongodb::bson::DateTime;
use serde::Deserialize;

mod register;

#[derive(Deserialize, Debug, Clone)]
pub struct UnregisteredEntry {
    pub name: String,
    // pub email: String,
    pub dob: DateTime,
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
