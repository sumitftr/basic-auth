use crate::AppError;
use lettre::{
    message::Mailbox,
    transport::smtp::authentication::Credentials,
    {Message, SmtpTransport, Transport},
};
use std::sync::LazyLock;

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
pub async fn send(to_email: &str, subject: String, body: String) -> Result<(), AppError> {
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
