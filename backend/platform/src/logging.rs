//! Structured logging init (task 2.2).
//!
//! `tracing` is the single logging API. This sets up the console subscriber
//! (env-filtered). The OpenTelemetry layers (traces + the logs signal) are added
//! in task group 6; secrets/tokens must never be logged (enforced by review +
//! the no-PII telemetry test, task 6.6).

use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};

/// Initialize the global tracing subscriber. Idempotent-safe: a second call is a
/// no-op (returns `false`).
pub fn init() -> bool {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,cymbra=debug"));
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .try_init()
        .is_ok()
}

/// Mask a `Bearer <token>` value so it can never leak into a log/span field
/// (task 6.6). Use whenever request metadata might be rendered.
pub fn redact_bearer(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut redact_next = false;
    for (i, tok) in input.split(' ').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        if redact_next && !tok.is_empty() {
            out.push_str("<redacted>");
            redact_next = false;
        } else {
            out.push_str(tok);
            if tok == "Bearer" {
                redact_next = true;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearer_token_never_survives_redaction() {
        let secret = "eyJ.header.signature-SECRET";
        let line = format!("authorization: Bearer {secret}");
        let red = redact_bearer(&line);
        assert!(red.contains("Bearer <redacted>"));
        assert!(!red.contains(secret), "token must not appear in output");
    }
}
