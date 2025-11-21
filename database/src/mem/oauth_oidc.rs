use common::oauth::OAuthProvider;
use std::sync::Arc;

// implementation block for those users who are authenticating using open_id_connect
impl crate::Db {
    #[inline]
    pub fn add_oauth_creds(
        self: &Arc<Self>,
        csrf_state: String,
        code_verifier: String,
        nonce: String,
        provider: OAuthProvider,
    ) {
        self.oauth_oidc
            .insert(csrf_state, (code_verifier, nonce, provider));
    }

    #[inline]
    pub fn get_oauth_creds(
        self: &Arc<Self>,
        csrf_state: &str,
    ) -> Option<(String, String, OAuthProvider)> {
        self.oauth_oidc.get(csrf_state)
    }

    #[inline]
    pub fn remove_oauth_creds(self: &Arc<Self>, csrf_state: &str) {
        self.oauth_oidc.remove(csrf_state);
    }
}
