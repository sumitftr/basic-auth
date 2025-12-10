use crate::AppError;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, message::Mailbox,
    transport::smtp::authentication::Credentials,
};
use std::sync::LazyLock;

// this is initialized in this static to not drop the connection after each mail send
static MAILER: LazyLock<AsyncSmtpTransport<Tokio1Executor>> = LazyLock::new(|| {
    let creds = Credentials::new(
        std::env::var("NOREPLY_EMAIL").unwrap(),
        std::env::var("SMTP_KEY").unwrap(),
    );
    // opening a remote connection to the mail server
    AsyncSmtpTransport::<Tokio1Executor>::relay(&std::env::var("SMTP_HOST").unwrap())
        .unwrap()
        .credentials(creds)
        .build()
});

// the noreply email is stored in this static variable to avoid parsing on every send
static NOREPLY_EMAIL: LazyLock<Mailbox> =
    LazyLock::new(|| std::env::var("NOREPLY_EMAIL").unwrap().parse().unwrap());

// Fire-and-forget version - returns immediately
pub fn send(to_email: String, subject: String, body: String) {
    tokio::spawn(async move {
        if let Err(e) = send_internal(to_email.clone(), subject, body).await {
            tracing::error!("Failed to send email to {}: {:?}", to_email, e);
        }
    });
}

// Wait for completion version - returns a JoinHandle you can await
pub fn send_async(
    to_email: String,
    subject: String,
    body: String,
) -> tokio::task::JoinHandle<Result<(), AppError>> {
    tokio::spawn(send_internal(to_email, subject, body))
}

// function to send any mail to the given mail address
pub async fn send_internal(
    to_email: String,
    subject: String,
    body: String,
) -> Result<(), AppError> {
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
        .send(msg)
        .await
        .map_err(|e| {
            tracing::error!("{e:?}");
            AppError::ServerError
        })
        .map(|_| ())
}
