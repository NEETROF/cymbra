//! Email-sender port (task 2.10): a trait so modules can send verification /
//! reset mail, with an SMTP impl for runtime and a [`FakeEmail`] for tests.

use crate::error::{AppError, Result};
use async_trait::async_trait;
use lettre::message::Mailbox;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::sync::Mutex;

/// Sends transactional email (verification, password reset).
#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send(&self, to: &str, subject: &str, body: &str) -> Result<()>;
}

/// SMTP-backed sender (Mailpit in dev, a real provider in prod).
pub struct SmtpSender {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl SmtpSender {
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
    async fn send(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        let to = to
            .parse::<Mailbox>()
            .map_err(|e| AppError::InvalidArgument(format!("invalid recipient: {e}")))?;
        let email = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject(subject)
            .body(body.to_string())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("build email: {e}")))?;
        self.transport
            .send(email)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("send email: {e}")))?;
        Ok(())
    }
}

/// Records sent messages for assertions in tests.
#[derive(Default)]
pub struct FakeEmail {
    pub sent: Mutex<Vec<(String, String, String)>>,
}

#[async_trait]
impl EmailSender for FakeEmail {
    async fn send(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        self.sent
            .lock()
            .unwrap()
            .push((to.into(), subject.into(), body.into()));
        Ok(())
    }
}
