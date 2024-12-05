use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub _id: ObjectId,
    pub name: String,
    pub email: String,
    pub gender: String,
    pub dob: DateTime,
    pub username: String,
    pub password: String,
    pub created: DateTime,
}

pub fn is_name_valid() -> bool {
    todo!()
}

pub fn is_email_valid() -> bool {
    todo!()
}

pub fn into_gender(value: &mut String) {
    let _ = value.to_lowercase();
    let _ = value.chars().next().unwrap().to_ascii_uppercase();
    if value != "Male" && value != "Female" {
        let _ = std::mem::replace(value, String::from("Other"));
    }
}

pub fn is_username_valid() -> bool {
    todo!()
}

pub fn is_date_valid() -> bool {
    todo!()
}
