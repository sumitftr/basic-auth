use hmac::{Hmac, Mac};
use lettre::{
    message::Mailbox,
    transport::smtp::authentication::Credentials,
    {Message, SmtpTransport, Transport},
};
use sha1::Sha1;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn generate_otp(secret: &[u8]) -> u32 {
    const SHA1_DIGEST_BYTES: usize = 20;

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
    let mut hasher: Hmac<Sha1> = Mac::new_from_slice(secret).unwrap();

    // hash the message
    hasher.update(message);

    // finalize the hash and convert to a static array
    let hash: [u8; SHA1_DIGEST_BYTES] = hasher.finalize().into_bytes().into();

    // calculate the dynamic offset for the value
    let dynamic_offset = (hash[SHA1_DIGEST_BYTES - 1] & (0xf_u8)) as usize;

    // build the u32 code from the hash
    ((hash[dynamic_offset] as u32) << 24
        | (hash[dynamic_offset + 1] as u32) << 16
        | (hash[dynamic_offset + 2] as u32) << 8
        | (hash[dynamic_offset + 3] as u32)) as u32
}

// this function is incomplete
pub async fn send_otp(to_email: &str, otp: u32) -> Result<(), String> {
    let smtp_key: &str = "<your-smtp-key>";
    let from_email: &str = "<your-email>";
    let smtp_host: &str = "smtp-relay.sendinblue.com";

    let creds: Credentials = Credentials::new(from_email.to_string(), smtp_key.to_string());

    // Open a remote connection
    let mailer: SmtpTransport = SmtpTransport::relay(&smtp_host)
        .unwrap()
        .credentials(creds)
        .build();

    let to_email: Mailbox = to_email.parse().map_err(|_| format!("Invalid Email"))?;
    let email: Message = Message::builder()
        .from(from_email.parse().unwrap())
        .to(to_email)
        .subject(format!("{otp} is your <SERVICE_NAME> verification code"))
        .body(format!(
            "Confirm your email address\n {otp}\n Thanks,\n <SERVICE_NAME>"
        ))
        .map_err(|e| e.to_string())?;

    // Send the email
    mailer.send(&email).map_err(|e| e.to_string()).map(|_| ())
}
