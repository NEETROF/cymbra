//! Cross-cutting error type and gRPC status mapping (task 2.4).

use tonic::{Code, Status};

/// Platform-wide result alias.
pub type Result<T> = std::result::Result<T, AppError>;

/// Errors surfaced across the backend. Each variant maps to a gRPC [`Status`]
/// via [`AppError::to_status`]; messages are safe to return to clients (no
/// secrets, no internal detail beyond the variant intent).
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Caller supplied invalid input.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    /// Authentication missing or failed.
    #[error("unauthenticated: {0}")]
    Unauthenticated(String),
    /// Authenticated but not permitted.
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    /// Resource not found (or not visible to the caller).
    #[error("not found: {0}")]
    NotFound(String),
    /// Uniqueness / already-exists conflict.
    #[error("already exists: {0}")]
    AlreadyExists(String),
    /// Optimistic-concurrency or state conflict.
    #[error("conflict: {0}")]
    Aborted(String),
    /// Precondition not met (e.g. email unverified).
    #[error("failed precondition: {0}")]
    FailedPrecondition(String),
    /// Rate limit / lockout hit.
    #[error("rate limited: {0}")]
    ResourceExhausted(String),
    /// Misconfiguration discovered at startup or runtime.
    #[error("configuration error: {0}")]
    Config(String),
    /// Unexpected internal failure (logged, not detailed to clients).
    #[error("internal error")]
    Internal(#[source] anyhow::Error),
}

impl AppError {
    /// Map to a gRPC [`Status`]. Internal errors collapse to a generic message.
    pub fn to_status(&self) -> Status {
        match self {
            AppError::InvalidArgument(m) => Status::new(Code::InvalidArgument, m.clone()),
            AppError::Unauthenticated(m) => Status::new(Code::Unauthenticated, m.clone()),
            AppError::PermissionDenied(m) => Status::new(Code::PermissionDenied, m.clone()),
            AppError::NotFound(m) => Status::new(Code::NotFound, m.clone()),
            AppError::AlreadyExists(m) => Status::new(Code::AlreadyExists, m.clone()),
            AppError::Aborted(m) => Status::new(Code::Aborted, m.clone()),
            AppError::FailedPrecondition(m) => Status::new(Code::FailedPrecondition, m.clone()),
            AppError::ResourceExhausted(m) => Status::new(Code::ResourceExhausted, m.clone()),
            AppError::Config(m) => Status::new(Code::Internal, format!("configuration error: {m}")),
            AppError::Internal(_) => Status::new(Code::Internal, "internal error"),
        }
    }
}

impl From<AppError> for Status {
    fn from(e: AppError) -> Self {
        e.to_status()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_variants_to_codes() {
        assert_eq!(
            AppError::Unauthenticated("x".into()).to_status().code(),
            Code::Unauthenticated
        );
        assert_eq!(
            AppError::Aborted("stale".into()).to_status().code(),
            Code::Aborted
        );
        // Internal errors never leak their source message.
        let s = AppError::Internal(anyhow::anyhow!("db secret leak")).to_status();
        assert_eq!(s.code(), Code::Internal);
        assert_eq!(s.message(), "internal error");
    }
}
