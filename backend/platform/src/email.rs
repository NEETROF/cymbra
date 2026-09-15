//! Email-sender port (task 2.10): a trait so modules can send verification /
//! reset mail, with an SMTP impl for runtime and a [`FakeEmail`] for tests.
//!
//! Messages are multipart/alternative — a plain-text part plus the branded HTML
//! produced by [`crate::email_template`] (change: template-backend-emails, design
//! D6). Producers render a [`RenderedEmail`] and hand the whole thing here; this
//! layer only transports it.

use crate::email_template::RenderedEmail;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use lettre::message::{Mailbox, MultiPart};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::sync::Mutex;
use std::time::Duration;

/// Upper bound on one whole SMTP send (connect, TLS, auth, envelope, data).
///
/// A send holds a job-runner slot in the worker and a request in the server.
/// lettre's own timeout applies per network operation (60 s each), so a relay that
/// stalls or trickles its replies could hold that slot for many minutes. Past this
/// bound the send fails and the job is retried; the rare cost is a duplicate
/// message when the relay had in fact accepted it.
pub const SEND_TIMEOUT: Duration = Duration::from_secs(30);

/// Sends transactional email (verification, password reset) as multipart HTML +
/// plain text.
#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send(&self, to: &str, email: &RenderedEmail) -> Result<()>;
}

/// SMTP-backed sender (Mailpit in dev, a real provider in prod).
pub struct SmtpSender {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
    send_timeout: Duration,
}

impl SmtpSender {
    /// `from` accepts a bare address or a display-name form parsed by `lettre`,
    /// e.g. `"Cymbra ID <no-reply@cymbra.app>"` (change: template-backend-emails).
    pub fn new(smtp_url: &str, from: &str) -> Result<Self> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::from_url(smtp_url)
            .map_err(|e| AppError::Config(format!("invalid SMTP url: {e}")))?
            .build();
        let from = from
            .parse::<Mailbox>()
            .map_err(|e| AppError::Config(format!("invalid SMTP from address: {e}")))?;
        Ok(Self {
            transport,
            from,
            send_timeout: SEND_TIMEOUT,
        })
    }
}

#[async_trait]
impl EmailSender for SmtpSender {
    async fn send(&self, to: &str, email: &RenderedEmail) -> Result<()> {
        let to = to
            .parse::<Mailbox>()
            .map_err(|e| AppError::InvalidArgument(format!("invalid recipient: {e}")))?;
        let message = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject(&email.subject)
            .multipart(MultiPart::alternative_plain_html(
                email.text.clone(),
                email.html.clone(),
            ))
            .map_err(|e| AppError::Internal(anyhow::anyhow!("build email: {e}")))?;
        tokio::time::timeout(self.send_timeout, self.transport.send(message))
            .await
            .map_err(|_| {
                AppError::Internal(anyhow::anyhow!(
                    "send email: timed out after {:?}",
                    self.send_timeout
                ))
            })?
            .map_err(|e| AppError::Internal(anyhow::anyhow!("send email: {e}")))?;
        Ok(())
    }
}

/// One message captured by [`FakeEmail`] for assertions in tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentEmail {
    pub to: String,
    pub subject: String,
    pub html: String,
    pub text: String,
}

/// Records sent messages for assertions in tests.
#[derive(Default)]
pub struct FakeEmail {
    pub sent: Mutex<Vec<SentEmail>>,
}

#[async_trait]
impl EmailSender for FakeEmail {
    async fn send(&self, to: &str, email: &RenderedEmail) -> Result<()> {
        self.sent.lock().unwrap().push(SentEmail {
            to: to.into(),
            subject: email.subject.clone(),
            html: email.html.clone(),
            text: email.text.clone(),
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rendered() -> RenderedEmail {
        RenderedEmail {
            subject: "s".into(),
            html: "<p>h</p>".into(),
            text: "t".into(),
        }
    }

    #[test]
    fn smtp_sender_accepts_display_name_from() {
        // The `"Name <addr>"` display-name form must parse (branded "Cymbra ID" sender).
        assert!(
            SmtpSender::new("smtp://localhost:1025", "Cymbra ID <no-reply@cymbra.app>").is_ok()
        );
    }

    #[test]
    fn smtp_sender_rejects_bad_from_and_url() {
        assert!(matches!(
            SmtpSender::new("smtp://localhost:1025", "not-an-email"),
            Err(AppError::Config(_))
        ));
        assert!(matches!(
            SmtpSender::new("://bad-url", "no-reply@cymbra.app"),
            Err(AppError::Config(_))
        ));
    }

    #[tokio::test]
    async fn smtp_send_gives_up_on_a_stalled_relay() {
        // Accepts the connection but never sends the SMTP greeting: without the
        // overall bound, lettre would wait its own 60 s per operation.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let mut open = Vec::new();
            while let Ok((socket, _)) = listener.accept().await {
                open.push(socket);
            }
        });
        let mut sender =
            SmtpSender::new(&format!("smtp://127.0.0.1:{port}"), "no-reply@cymbra.app").unwrap();
        sender.send_timeout = Duration::from_millis(200);

        let started = std::time::Instant::now();
        let err = sender.send("to@x.dev", &rendered()).await.unwrap_err();
        assert!(matches!(err, AppError::Internal(_)), "{err:?}");
        assert!(format!("{err:?}").contains("timed out"), "{err:?}");
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[tokio::test]
    async fn fake_email_records_multipart_message() {
        let fake = FakeEmail::default();
        fake.send("to@x.dev", &rendered()).await.unwrap();
        let sent = fake.sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].to, "to@x.dev");
        assert_eq!(sent[0].subject, "s");
        assert_eq!(sent[0].html, "<p>h</p>");
        assert_eq!(sent[0].text, "t");
    }
}
