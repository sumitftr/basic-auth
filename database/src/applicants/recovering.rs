use super::PasswordResetInfo;
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
        tracing::info!("[Password Reset Request] Email: {email}, Code: {code}");
        self.recovering.insert(code, PasswordResetInfo { email, socket_addr });
    }

    // updates password of the given user (returns email)
    pub async fn reset_password(
        self: &Arc<Self>,
        socket_addr: SocketAddr,
        code: &str,
        password: &str,
    ) -> Result<String, AppError> {
        match self.recovering.get(code) {
            Some(entry) => {
                if entry.socket_addr.ip() != socket_addr.ip() {
                    return Err(AppError::Unauthorized("IP Address mismatch"));
                }
                self.update_password(&entry.email, password).await?;
                self.recovering.remove(code);
                tracing::info!("[Password Reset] Email: {}, Password: {password}", &entry.email);
                Ok(entry.email)
            }
            None => Err(AppError::BadReq("Password Reset code not found")),
        }
    }
}
