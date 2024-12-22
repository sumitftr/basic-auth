use crate::models::user::*;
use mongodb::bson::{oid::ObjectId, DateTime};
use std::collections::hash_map::Entry;
use std::sync::Arc;

// sub steps for registering an user
impl super::DBConf {
    pub async fn create_user(
        self: Arc<Self>,
        name: String,
        email: String,
        day: u8,
        month: u8,
        year: i32,
    ) -> Result<(), String> {
        let name = is_name_valid(&name)?;
        if !is_email_valid(&email) {
            return Err("Invalid Email Format".to_string());
        };
        // checking if the email is already used or not
        if let Err(e) = self.is_email_available(&email).await {
            if let Some(s) = e.get_custom::<&str>() {
                return Err(s.to_string());
            }
            return Err(e.to_string());
        }
        // checking if the date of birth is valid or not
        let dob = match DateTime::builder().year(year).month(month).day(day).build() {
            Ok(v) if v > DateTime::now() => return Err("Invalid Date of Birth".to_string()),
            Ok(v) => v,
            Err(e) => return Err(e.to_string()),
        };
        let mut guard = self.registration.lock().unwrap();
        guard.insert(
            email.clone(),
            RegisterUser {
                name,
                email,
                dob,
                password: None,
                username: None,
                register_status: RegisterStatus::Created,
            },
        );
        Ok(())
    }

    pub fn verify_email(self: Arc<Self>, email: String, otp: u32) -> Result<(), String> {
        let mut guard = self.registration.lock().unwrap();
        let entry = guard.entry(email);
        if let Entry::Occupied(mut v) = entry {
            if v.get().register_status != RegisterStatus::Created {
                return Err("OTP can't be sent multiple times".to_string());
            }
            // not finished
            // otp checking
            v.get_mut().register_status = RegisterStatus::EmailVerified;
        } else {
            return Err("User not found".to_string());
        }
        Ok(())
    }

    pub fn set_password(self: Arc<Self>, email: String, password: String) -> Result<(), String> {
        let mut guard = self.registration.lock().unwrap();
        let entry = guard.entry(email);
        if let Entry::Occupied(mut v) = entry {
            if v.get().register_status != RegisterStatus::EmailVerified {
                return Err("User email not verified".to_string());
            }
            if password.len() < 8 {
                return Err("Password should be of atleast 8 characters".to_string());
            }
            v.get_mut().password = Some(password);
            v.get_mut().register_status = RegisterStatus::PasswordSet;
        } else {
            return Err("User not found".to_string());
        }
        Ok(())
    }

    pub async fn register(self: Arc<Self>, email: String, username: String) -> Result<(), String> {
        let value: RegisterUser;

        let mut guard = self.registration.lock().unwrap();
        let entry = guard.entry(email);
        if let Entry::Occupied(v) = entry {
            if v.get().register_status != RegisterStatus::PasswordSet {
                return Err("User password not set".to_string());
            }
            // have to use tokio::sync::Mutex for this
            is_username_valid(&username)?;
            if let Err(e) = self.is_username_available(&username).await {
                if let Some(s) = e.get_custom::<&str>() {
                    return Err(s.to_string());
                }
                return Err(e.to_string());
            }
            value = v.remove_entry().1;
        } else {
            return Err("User not found".to_string());
        }
        drop(guard);

        self.add_user(&crate::models::User {
            _id: ObjectId::new(),
            name: value.name,
            email: value.email,
            dob: value.dob,
            password: value.password.unwrap(),
            username,
            gender: None,
            phone: None,
            created: DateTime::now(),
            last_login: DateTime::now(),
        });
        Ok(())
    }
}
