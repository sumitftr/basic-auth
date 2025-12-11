use crate::applicant::OAuthInfo;
use common::oauth::OAuthProvider;
use std::{net::SocketAddr, sync::Arc};

// implementation block for those users who are authenticating using open_id_connect
impl crate::Db {
    #[inline]
    pub fn add_oauth_creds(
        self: &Arc<Self>,
        socket_addr: SocketAddr,
        csrf_state: String,
        code_verifier: String,
        nonce: String,
        provider: OAuthProvider,
    ) {
        let oauth_info = OAuthInfo { csrf_state, code_verifier, nonce, provider };
        self.oauth_oidc.insert(socket_addr, oauth_info);
    }

    #[inline]
    pub fn get_oauth_creds(self: &Arc<Self>, socket_addr: &SocketAddr) -> Option<OAuthInfo> {
        self.oauth_oidc.get(socket_addr)
    }

    #[inline]
    pub fn remove_oauth_creds(self: &Arc<Self>, socket_addr: &SocketAddr) -> Option<OAuthInfo> {
        self.oauth_oidc.remove(socket_addr)
    }
}
