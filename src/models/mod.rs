use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub _id: ObjectId,
    pub name: String,
    pub email: String,
    pub gender: Gender,
    pub dob: DateTime,
    pub username: String,
    pub password: String,
    pub created: DateTime,
}

#[derive(Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Other,
}

impl std::convert::From<String> for Gender {
    fn from(value: String) -> Self {
        if value == "Male" || value == "male" {
            Gender::Male
        } else if value == "Female" || value == "female" {
            Gender::Female
        } else {
            Gender::Other
        }
    }
}
