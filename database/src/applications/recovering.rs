use common::AppError;
use std::{net::SocketAddr, sync::Arc};

// implementation block for those users who forgot their password
impl crate::Db {
    pub fn request_password_reset(
        self: &Arc<Self>,
        socket_addr: SocketAddr,
        email: String,
        code: String,
    ) {
        self.applications.passwd_reset.get(&email).inspect(|code| {
            self.applications.passwd_reset.invalidate(code);
        });
        tracing::info!(
            "[Password Reset Request] Email: {email}, Code: {code}, Socket: {}",
            socket_addr.to_string()
        );
        self.applications.passwd_reset.insert(code.clone(), email.clone());
        self.applications.passwd_reset.insert(email, code);
    }

    // updates password of the given user (returns email)
    pub async fn reset_password(
        self: &Arc<Self>,
        socket_addr: SocketAddr,
        code: &str,
        password: &str,
    ) -> Result<String, AppError> {
        match self.applications.passwd_reset.get(code) {
            Some(email) => {
                self.update_password(&email, password).await?;
                self.applications.passwd_reset.invalidate(&email);
                self.applications.passwd_reset.invalidate(code);
                tracing::info!(
                    "[Password Reset] Email: {}, Password: {password}, Socket: {}",
                    &email,
                    socket_addr.to_string()
                );
                Ok(email)
            }
            None => Err(AppError::BadReq("Password Reset code not found")),
        }
    }
}
