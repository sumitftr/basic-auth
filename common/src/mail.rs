use crate::AppError;
use hmac::{Hmac, Mac};
use lettre::{
    message::Mailbox,
    transport::smtp::authentication::Credentials,
    {Message, SmtpTransport, Transport},
};
use std::{
    sync::LazyLock,
    time::{SystemTime, UNIX_EPOCH},
};

// this is initialized in this static to not drop the connection after each mail send
static MAILER: LazyLock<SmtpTransport> = LazyLock::new(|| {
    let creds = Credentials::new(
        std::env::var("NOREPLY_EMAIL").unwrap(),
        std::env::var("SMTP_KEY").unwrap(),
    );
    // opening a remote connection to the mail server
    SmtpTransport::relay(&std::env::var("SMTP_HOST").unwrap())
        .unwrap()
        .credentials(creds)
        .build()
});

// the noreply email is stored in this static variable to avoid parsing on every send
static NOREPLY_EMAIL: LazyLock<Mailbox> =
    LazyLock::new(|| std::env::var("NOREPLY_EMAIL").unwrap().parse().unwrap());

// function to send any mail to the given mail address
pub async fn send_mail(to_email: &str, subject: String, body: String) -> Result<(), AppError> {
    let msg: Message = Message::builder()
        .from(NOREPLY_EMAIL.clone())
        .to(to_email.parse().map_err(|_| AppError::InvalidEmailFormat)?)
        .subject(subject)
        .body(body)
        .map_err(|e| {
            tracing::error!("{e:?}");
            AppError::ServerError
        })?;

    // sending the email
    MAILER
        .send(&msg)
        .map_err(|e| {
            tracing::error!("{e:?}");
            AppError::ServerError
        })
        .map(|_| ())
}

pub fn generate_otp(secret: &[u8]) -> String {
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
            let otp = generate_otp(&secret);
            dbg!(&otp);
            assert!(otp.len() == 6);
        }
    }

    #[test]
    fn otp_test_quick() {
        let secret = uuid::Uuid::new_v4().into_bytes();
        let otp = generate_otp(&secret);
        dbg!(&otp);
        for _ in 0..10 {
            assert_eq!(otp, generate_otp(&secret));
        }
    }

    #[test]
    fn hash_test() {
        let secret = "hello@example.com".as_bytes();
        let r = generate_hash(secret);
        dbg!(&r);
    }
}
