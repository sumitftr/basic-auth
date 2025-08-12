use crate::user::RegisterStatus;
use common::{AppError, validation};
use mongodb::bson::{DateTime, oid::ObjectId};
use std::collections::hash_map::Entry;
use std::sync::Arc;

// sub steps for registering an user
impl crate::Db {
    pub async fn create_user(
        self: Arc<Self>,
        name: String,
        email: String,
        day: u8,
        month: u8,
        year: i32,
    ) -> Result<(), AppError> {
        let name = validation::is_name_valid(&name)?;
        if !validation::is_email_valid(&email) {
            return Err(AppError::BadReq("Invalid Email Format"));
        };
        // checking if the email is already used or not
        self.is_email_available(&email).await?;
        // checking if the date of birth is valid or not
        let dob = match DateTime::builder().year(year).month(month).day(day).build() {
            Ok(v) if v > DateTime::now() => return Err(AppError::BadReq("Invalid Date of Birth")),
            Ok(v) => v,
            Err(e) => {
                tracing::error!("{e:?}");
                return Err(AppError::ServerDefault);
            }
        };
        // generating otp
        let otp = common::mail::generate_otp(email.as_bytes());

        let mut guard = self.unregistered.lock().unwrap();
        guard.insert(
            email,
            crate::user::UnregisteredEntry {
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

    pub async fn resend_otp(self: Arc<Self>, email: String) -> Result<(), AppError> {
        // generating otp
        let otp = common::mail::generate_otp(email.as_bytes());

        let mut guard = self.unregistered.lock().unwrap();
        let entry = guard.entry(email);
        if let Entry::Occupied(mut v) = entry {
            if v.get().register_status != RegisterStatus::Created {
                return Err(AppError::BadReq("OTP can't be sent multiple times"));
            }
            v.get_mut().otp = otp;
        } else {
            return Err(AppError::BadReq("User not found"));
        }
        drop(guard);

        // sending mail

        Ok(())
    }

    pub fn verify_email(self: Arc<Self>, email: String, otp: u32) -> Result<(), AppError> {
        let mut guard = self.unregistered.lock().unwrap();
        let entry = guard.entry(email);
        if let Entry::Occupied(mut v) = entry {
            if v.get().register_status != RegisterStatus::Created {
                return Err(AppError::BadReq("OTP can't be sent multiple times"));
            }
            if v.get().otp != otp {
                return Err(AppError::BadReq("OTP doesn't match"));
            }
            v.get_mut().register_status = RegisterStatus::EmailVerified;
        } else {
            return Err(AppError::UserNotFound);
        }
        Ok(())
    }

    pub fn set_password(self: Arc<Self>, email: String, password: String) -> Result<(), AppError> {
        let mut guard = self.unregistered.lock().unwrap();
        let entry = guard.entry(email);
        if let Entry::Occupied(mut v) = entry {
            if v.get().register_status != RegisterStatus::EmailVerified {
                return Err(AppError::BadReq("User email not verified"));
            }
            if password.len() < 8 {
                return Err(AppError::BadReq(
                    "Password should be of atleast 8 characters",
                ));
            }
            v.get_mut().password = Some(password);
            v.get_mut().register_status = RegisterStatus::PasswordSet;
        } else {
            return Err(AppError::UserNotFound);
        }
        Ok(())
    }

    pub async fn set_username(
        self: Arc<Self>,
        email: String,
        username: String,
    ) -> Result<crate::user::User, AppError> {
        validation::is_username_valid(&username)?;
        self.is_username_available(&username).await?;

        let metadata: crate::user::UnregisteredEntry;
        let user_email: String;

        let mut guard = self.unregistered.lock().unwrap();
        let entry = guard.entry(email);
        if let Entry::Occupied(v) = entry {
            if v.get().register_status != RegisterStatus::PasswordSet {
                return Err(AppError::BadReq("User password not set"));
            }
            (user_email, metadata) = v.remove_entry();
        } else {
            return Err(AppError::UserNotFound);
        }
        drop(guard);

        Ok(crate::user::User {
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
