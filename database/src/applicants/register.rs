use crate::{
    applicants::{ApplicantEntry, RegisterStatus},
    user::User,
};
use common::{AppError, user_session::UserSession, validation};
use mongodb::bson::{DateTime, oid::ObjectId};
use std::sync::Arc;

// sub steps for registering an user
impl crate::Db {
    pub async fn create_applicant(
        self: Arc<Self>,
        name: String,
        email: String,
        day: u8,
        month: u8,
        year: u32,
    ) -> Result<(), AppError> {
        // checking whether the name and email is valid or not
        let name = validation::is_name_valid(&name)?;
        validation::is_email_valid(&email)?;

        // checking if the email is already used or not
        self.is_email_available(&email).await?;

        // checking if the birth date is valid or not
        let birth_date = common::validation::is_birth_date_valid(year, month, day)?;

        // generating otp
        let otp = common::mail::generate_otp(email.as_bytes());
        tracing::info!("OTP: {otp}, Email: {email}");

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

        // inserting `ApplicantEntry` to memory store
        self.applicants.insert(
            email,
            ApplicantEntry {
                name,
                birth_date,
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

        if let Some(mut entry) = self.applicants.get(&email) {
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
            self.applicants.insert(email, entry);
        } else {
            return Err(AppError::UserNotFound);
        }

        Ok(())
    }

    pub async fn verify_email(self: Arc<Self>, email: String, otp: String) -> Result<(), AppError> {
        if let Some(mut entry) = self.applicants.get(&email) {
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
            self.applicants.insert(email, entry);
        } else {
            return Err(AppError::UserNotFound);
        }

        Ok(())
    }

    pub fn set_password(self: Arc<Self>, email: String, password: String) -> Result<(), AppError> {
        if let Some(mut entry) = self.applicants.get(&email) {
            if entry.register_status != RegisterStatus::EmailVerified {
                return Err(AppError::BadReq("User email not verified"));
            }
            common::validation::is_password_valid(&password)?;
            entry.password = Some(password);
            entry.register_status = RegisterStatus::PasswordSet;
            self.applicants.insert(email, entry);
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

        let metadata: ApplicantEntry;

        if let Some(v) = self.applicants.get(&email) {
            if v.register_status != RegisterStatus::PasswordSet {
                return Err(AppError::BadReq("User password not set"));
            }
            // removing the `UnregisteredEntry` from `Db::unregistered`
            // SAFETY: This will not panic since the entry is already present
            metadata = self.applicants.remove(&email).unwrap();
        } else {
            return Err(AppError::UserNotFound);
        }

        let user = User {
            _id: ObjectId::new(),
            legal_name: metadata.name.clone(),
            email,
            birth_date: metadata.birth_date,
            password: metadata.password.unwrap(),
            username,
            profile_pic: None,
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
