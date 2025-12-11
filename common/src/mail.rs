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

// function to send any mail to the given mail address
pub async fn send(to_email: String, subject: String, body: String) -> Result<(), AppError> {
    let msg: Message = Message::builder()
        .from(NOREPLY_EMAIL.clone())
        .to(to_email.parse().map_err(|_| AppError::InvalidEmailFormat)?)
        .subject(subject)
        .body(body)
        .map_err(|e| {
            tracing::error!("{e:?}");
            AppError::ServerError
        })?;

    // // sending the email and wait for completion
    // tokio::spawn(async move {
    //     MAILER
    //         .send(msg)
    //         .await
    //         .map_err(|e| {
    //             tracing::error!("{e:?}");
    //             AppError::ServerError
    //         })
    //         .map(|_| ())
    // })
    // .await
    // .map_err(|e| {
    //     tracing::error!("Task join error: {e:?}");
    //     AppError::ServerError
    // })?

    // Fire-and-forget version - returns immediately
    tokio::spawn(async move {
        if let Err(e) = MAILER.send(msg).await {
            tracing::error!("Failed to send email: {e:?}");
        }
    });
    Ok(())
}
