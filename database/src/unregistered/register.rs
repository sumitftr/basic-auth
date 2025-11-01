use crate::{
    unregistered::{RegisterStatus, UnregisteredEntry},
    user::User,
};
use common::{AppError, user_session::UserSession, validation};
use mongodb::bson::{DateTime, oid::ObjectId};
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
        // checking whether the name and email is valid or not
        let name = validation::is_name_valid(&name)?;
        validation::is_email_valid(&email)?;

        // checking if the email is already used or not
        self.is_email_available(&email).await?;

        // checking if the date of birth is valid or not
        let dob = match DateTime::builder().year(year).month(month).day(day).build() {
            Ok(v) if v > DateTime::now() => return Err(AppError::BadReq("Invalid Date of Birth")),
            Ok(v) => v,
            Err(e) => {
                tracing::error!("{e:?}");
                return Err(AppError::ServerError);
            }
        };

        // generating otp
        let otp = common::mail::generate_otp(email.as_bytes());

        // sending otp to the email
        common::mail::send_mail(
            email.as_str(),
            format!("{otp} is your {} verification code", &*common::SERVICE_NAME),
            format!(
                "Confirm your email address\n {otp}\n Thanks,\n {}",
                &*common::SERVICE_NAME
            ),
        )
        .await?;

        // inserting `UnregisteredEntry` to memory store
        self.unregistered.insert(
            email,
            UnregisteredEntry {
                name,
                dob,
                otp,
                password: None,
                register_status: RegisterStatus::Created,
                session: vec![],
            },
        );

        Ok(())
    }

    pub async fn resend_otp(self: Arc<Self>, email: String) -> Result<(), AppError> {
        // generating otp
        let otp = common::mail::generate_otp(email.as_bytes());

        if let Some(mut entry) = self.unregistered.get(&email) {
            if entry.register_status != RegisterStatus::Created {
                return Err(AppError::BadReq("OTP can't be sent multiple times"));
            }
            entry.otp = otp.clone();
            // resending otp to the email
            common::mail::send_mail(
                email.as_str(),
                format!("{otp} is your {} verification code", &*common::SERVICE_NAME),
                format!(
                    "Confirm your email address\n {otp}\n Thanks,\n {}",
                    &*common::SERVICE_NAME
                ),
            )
            .await?;
            // inserting new entry to memory store
            self.unregistered.insert(email, entry);
        } else {
            return Err(AppError::UserNotFound);
        }

        Ok(())
    }

    pub async fn verify_email(self: Arc<Self>, email: String, otp: String) -> Result<(), AppError> {
        if let Some(mut entry) = self.unregistered.get(&email) {
            if entry.register_status != RegisterStatus::Created {
                return Err(AppError::BadReq("OTP can't be sent multiple times"));
            }
            if entry.otp != otp {
                return Err(AppError::BadReq("OTP doesn't match"));
            }
            entry.register_status = RegisterStatus::EmailVerified;
            // sending email verification success
            common::mail::send_mail(
                email.as_str(),
                format!("Your email {email} has been verified successfully"),
                format!(
                    "Your email {email} has been verified successfully\n Thanks,\n {}",
                    &*common::SERVICE_NAME
                ),
            )
            .await?;
            // inserting new entry to memory store
            self.unregistered.insert(email, entry);
        } else {
            return Err(AppError::UserNotFound);
        }

        Ok(())
    }

    pub fn set_password(self: Arc<Self>, email: String, password: String) -> Result<(), AppError> {
        if let Some(mut entry) = self.unregistered.get(&email) {
            if entry.register_status != RegisterStatus::EmailVerified {
                return Err(AppError::BadReq("User email not verified"));
            }
            if password.len() < 8 {
                return Err(AppError::BadReq(
                    "Password should be of atleast 8 characters",
                ));
            }
            entry.password = Some(password);
            entry.register_status = RegisterStatus::PasswordSet;
            self.unregistered.insert(email, entry);
        } else {
            return Err(AppError::UserNotFound);
        }
        Ok(())
    }

    pub async fn set_username(
        self: Arc<Self>,
        email: String,
        username: String,
        new_session: UserSession,
    ) -> Result<User, AppError> {
        validation::is_username_valid(&username)?;

        // checking if the username is already used or not
        self.is_username_available(&username).await?;

        let metadata: UnregisteredEntry;

        if let Some(v) = self.unregistered.get(&email) {
            if v.register_status != RegisterStatus::PasswordSet {
                return Err(AppError::BadReq("User password not set"));
            }
            // removing the `UnregisteredEntry` from `Db::unregistered`
            // SAFETY: This will not panic since the entry is already present
            metadata = self.unregistered.remove(&email).unwrap();
        } else {
            return Err(AppError::UserNotFound);
        }

        let user = User {
            _id: ObjectId::new(),
            legal_name: metadata.name.clone(),
            email,
            dob: metadata.dob,
            password: metadata.password.unwrap(),
            username,
            display_name: metadata.name,
            bio: None,
            gender: None,
            phone: None,
            country: None,
            sessions: vec![new_session],
            created: DateTime::now(),
            // last_login: DateTime::now(),
        };
        self.add_user(&user).await?;

        Ok(user)
    }
}
