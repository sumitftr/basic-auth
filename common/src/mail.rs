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

pub fn generate_otp(secret: &[u8]) -> u32 {
    const SHA256_DIGEST_BYTES: usize = 32;

    let counter = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64;

    // build hmac key from counter
    let message: &[u8; 8] = &[
        ((counter >> 56) & 0xff) as u8,
        ((counter >> 48) & 0xff) as u8,
        ((counter >> 40) & 0xff) as u8,
        ((counter >> 32) & 0xff) as u8,
        ((counter >> 24) & 0xff) as u8,
        ((counter >> 16) & 0xff) as u8,
        ((counter >> 8) & 0xff) as u8,
        ((counter) & 0xff) as u8,
    ];

    // Create the hasher with the key. We can use expect for Hmac algorithms as they allow arbitrary key sizes.
    let mut hasher: Hmac<sha2::Sha256> = Mac::new_from_slice(secret).unwrap();

    // hash the message
    hasher.update(message);

    // finalize the hash and convert to a static array
    let hash = hasher.finalize().into_bytes();

    // calculate the dynamic offset for the value
    let dynamic_offset = (hash[SHA256_DIGEST_BYTES - 1] & (0xf_u8)) as usize;

    // build the u32 code from the hash
    ((hash[dynamic_offset] as u32) << 24
        | (hash[dynamic_offset + 1] as u32) << 16
        | (hash[dynamic_offset + 2] as u32) << 8
        | (hash[dynamic_offset + 3] as u32)) as u32
}
