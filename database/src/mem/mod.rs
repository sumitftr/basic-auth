mod active;
mod openid_connecting;
mod recovering;

#[derive(Clone)]
pub struct OAuthInfo {
    pub csrf_state: String,
    pub code_verifier: String,
    pub nonce: String,
    pub provider: common::oauth::OAuthProvider,
}

#[derive(Clone)]
pub struct PasswordResetInfo {
    pub email: String,
    pub socket_addr: std::net::SocketAddr,
}
