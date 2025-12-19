use super::OidcInfo;
use common::oauth::OAuthProvider;
use std::{net::SocketAddr, sync::Arc};

// implementation block for those users who are authenticating using open_id_connect
impl crate::Db {
    #[inline]
    pub fn add_oidc_info(
        self: &Arc<Self>,
        socket_addr: SocketAddr,
        csrf_state: String,
        code_verifier: String,
        nonce: String,
        provider: OAuthProvider,
    ) {
        let oauth_info = OidcInfo { socket_addr, code_verifier, nonce, provider };
        self.applications.oidconnect.insert(csrf_state, oauth_info);
    }

    #[inline]
    pub fn get_oidc_info(self: &Arc<Self>, csrf_state: &str) -> Option<OidcInfo> {
        self.applications.oidconnect.get(csrf_state)
    }

    #[inline]
    pub fn remove_oidc_info(self: &Arc<Self>, csrf_state: &str) -> Option<OidcInfo> {
        self.applications.oidconnect.remove(csrf_state)
    }
}
