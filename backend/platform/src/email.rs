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
        Ok(Self { transport, from })
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
        self.transport
            .send(message)
            .await
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
