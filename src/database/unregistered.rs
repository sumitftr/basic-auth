use crate::models::user::RegisterStatus;
use crate::utils::validation;
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
        let name = validation::is_name_valid(&name)?;
        if !validation::is_email_valid(&email) {
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
        // generating otp
        let otp = crate::utils::mail::generate_otp(email.as_bytes());

        let mut guard = self.unregistered.lock().unwrap();
        guard.insert(
            email,
            crate::models::user::UnregisteredEntry {
                name,
                dob,
                otp,
                password: None,
                register_status: RegisterStatus::Created,
            },
        );
        drop(guard);

        // sending mail

        Ok(())
    }

    pub async fn resend_otp(self: Arc<Self>, email: String) -> Result<(), String> {
        // generating otp
        let otp = crate::utils::mail::generate_otp(email.as_bytes());

        let mut guard = self.unregistered.lock().unwrap();
        let entry = guard.entry(email);
        if let Entry::Occupied(mut v) = entry {
            if v.get().register_status != RegisterStatus::Created {
                return Err("OTP can't be sent multiple times".to_string());
            }
            v.get_mut().otp = otp;
        } else {
            return Err("User not found".to_string());
        }
        drop(guard);

        // sending mail

        Ok(())
    }

    pub fn verify_email(self: Arc<Self>, email: String, otp: u32) -> Result<(), String> {
        let mut guard = self.unregistered.lock().unwrap();
        let entry = guard.entry(email);
        if let Entry::Occupied(mut v) = entry {
            if v.get().register_status != RegisterStatus::Created {
                return Err("OTP can't be sent multiple times".to_string());
            }
            if v.get().otp != otp {
                return Err("OTP doesn't match".to_string());
            }
            v.get_mut().register_status = RegisterStatus::EmailVerified;
        } else {
            return Err("User not found".to_string());
        }
        Ok(())
    }

    pub fn set_password(self: Arc<Self>, email: String, password: String) -> Result<(), String> {
        let mut guard = self.unregistered.lock().unwrap();
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

    pub async fn set_username(
        self: Arc<Self>,
        email: String,
        username: String,
    ) -> Result<crate::models::user::User, String> {
        validation::is_username_valid(&username)?;
        if let Err(e) = self.is_username_available(&username).await {
            if let Some(s) = e.get_custom::<&str>() {
                return Err(s.to_string());
            }
            return Err("Something went wrong".to_string());
        }

        let metadata: crate::models::user::UnregisteredEntry;
        let user_email: String;

        let mut guard = self.unregistered.lock().unwrap();
        let entry = guard.entry(email);
        if let Entry::Occupied(v) = entry {
            if v.get().register_status != RegisterStatus::PasswordSet {
                return Err("User password not set".to_string());
            }
            (user_email, metadata) = v.remove_entry();
        } else {
            return Err("User not found".to_string());
        }
        drop(guard);

        Ok(crate::models::user::User {
            _id: ObjectId::new(),
            legal_name: metadata.name.clone(),
            email: user_email,
            dob: metadata.dob,
            password: metadata.password.unwrap(),
            username,
            display_name: metadata.name,
            bio: None,
            gender: None,
            phone: None,
            country: None,
            // created: DateTime::now(),
            // last_login: DateTime::now(),
        })
    }
}
