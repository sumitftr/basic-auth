use hmac::{Hmac, Mac};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate(secret: &[u8]) -> String {
    const TIME_STEP: u64 = 30; // 30 seconds
    const DIGITS_POWER: u32 = 1_000_000; // 10^6

    // Get current time in seconds, divided into 30-second intervals
    let counter = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX_EPOCH")
        .as_secs()
        / TIME_STEP;

    // Convert counter to big-endian byte array
    let message: [u8; 8] = [
        (counter >> 56) as u8,
        (counter >> 48) as u8,
        (counter >> 40) as u8,
        (counter >> 32) as u8,
        (counter >> 24) as u8,
        (counter >> 16) as u8,
        (counter >> 8) as u8,
        counter as u8,
    ];

    // HMAC-SHA256
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret).unwrap();
    mac.update(&message);
    let result = mac.finalize().into_bytes();

    // Dynamic truncation
    let offset = (result[result.len() - 1] & 0xf) as usize;
    let binary = ((result[offset] as u32 & 0x7f) << 24)
        | ((result[offset + 1] as u32) << 16)
        | ((result[offset + 2] as u32) << 8)
        | (result[offset + 3] as u32);

    // Truncate to 6 digits
    let otp = binary % DIGITS_POWER;

    // Return as zero-padded 6-digit string
    format!("{:06}", otp)
}

pub fn generate_hash(secret: &[u8]) -> String {
    const TIME_STEP: u64 = 30; // 30-second windows

    // ---- Counter (big-endian, 8 bytes) ----
    let counter = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before UNIX_EPOCH")
        .as_secs()
        / TIME_STEP;

    let counter_bytes = counter.to_be_bytes(); // network order

    // ---- HMAC-SHA-256 ----
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret).unwrap();
    mac.update(&counter_bytes);
    let result = mac.finalize().into_bytes();

    // ---- Hex-encode (64 characters) ----
    const_hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otp_test() {
        for _ in 0..10 {
            let secret = uuid::Uuid::new_v4().into_bytes();
            let otp = generate(&secret);
            dbg!(&otp);
            assert!(otp.len() == 6);
        }
    }

    #[test]
    fn otp_test_quick() {
        let secret = uuid::Uuid::new_v4().into_bytes();
        let otp = generate(&secret);
        dbg!(&otp);
        for _ in 0..10 {
            assert_eq!(otp, generate(&secret));
        }
    }

    #[test]
    fn hash_test() {
        let secret = "hello@example.com".as_bytes();
        let r = generate_hash(secret);
        dbg!(&r);
    }
}
