use base64::Engine;
use hmac::Mac;

pub const BASE64_DIGEST_LEN: usize = 44;

/// this function is used to sign cookie value to ensure integrity and authenticity
///
/// value is the `VALUE` part of the whole cookie (`KEY=VALUE`)
pub fn sign(value: &str) -> String {
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(&crate::SECRET_KEY).unwrap();
    hmac::Mac::update(&mut mac, value.as_bytes());
    base64::prelude::BASE64_STANDARD.encode(mac.finalize().into_bytes())
}

/// this function is used to verify signed cookie value to ensure integrity and authenticity
///
/// value is the `VALUE` part of the whole cookie (`KEY=VALUE`)
pub fn verify(value: &str) -> Option<String> {
    if !value.is_char_boundary(BASE64_DIGEST_LEN) {
        return None;
    }

    // Split [MAC | original-value] into its two parts.
    let (digest_str, uid) = value.split_at(BASE64_DIGEST_LEN);
    let digest = base64::prelude::BASE64_STANDARD.decode(digest_str).ok()?;

    // Perform the verification
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(&crate::SECRET_KEY).unwrap();
    hmac::Mac::update(&mut mac, uid.as_bytes());
    mac.verify_slice(&digest).map(|_| uid.to_string()).ok()
}
