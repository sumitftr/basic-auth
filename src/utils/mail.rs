use hmac::{Hmac, Mac};
use sha1::Sha1;

pub fn generate_otp(secret: &[u8]) -> u32 {
    const SHA1_DIGEST_BYTES: usize = 20;

    let counter = chrono::Utc::now().timestamp() as u64;

    // build hmac key from counter
    let message: &[u8; 8] = &[
        ((counter >> 56) & 0xff) as u8,
        ((counter >> 48) & 0xff) as u8,
        ((counter >> 40) & 0xff) as u8,
        ((counter >> 32) & 0xff) as u8,
        ((counter >> 24) & 0xff) as u8,
        ((counter >> 16) & 0xff) as u8,
        ((counter >> 8) & 0xff) as u8,
        ((counter >> 0) & 0xff) as u8,
    ];

    // Create the hasher with the key. We can use expect for Hmac algorithms as they allow arbitrary key sizes.
    let mut hasher: Hmac<Sha1> = Mac::new_from_slice(secret).unwrap();

    // hash the message
    hasher.update(message);

    // finalize the hash and convert to a static array
    let hash: [u8; SHA1_DIGEST_BYTES] = hasher.finalize().into_bytes().into();

    // calculate the dynamic offset for the value
    let dynamic_offset = (hash[SHA1_DIGEST_BYTES - 1] & (0xf as u8)) as usize;

    // build the u32 code from the hash
    ((hash[dynamic_offset] as u32) << 24
        | (hash[dynamic_offset + 1] as u32) << 16
        | (hash[dynamic_offset + 2] as u32) << 8
        | (hash[dynamic_offset + 3] as u32)) as u32
}
