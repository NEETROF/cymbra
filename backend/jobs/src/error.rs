//! The job substrate's error type. Kept self-contained (no dependency on
//! `cymbra-platform`) so the engine stays swappable/extractable; producers map
//! it into their own error type at the call site.

/// Result alias for job-substrate operations.
pub type Result<T> = std::result::Result<T, JobError>;

/// Failures surfaced by enqueue / scheduling / dead-lettering.
#[derive(Debug, thiserror::Error)]
pub enum JobError {
    /// A schedule's cron expression or timezone could not be parsed.
    #[error("invalid schedule {name:?}: {reason}")]
    InvalidSchedule { name: String, reason: String },
    /// The job payload could not be (de)serialized.
    #[error("payload error: {0}")]
    Payload(String),
    /// The underlying database / queue engine failed.
    #[error("queue engine error")]
    Engine(#[source] anyhow::Error),
}

impl From<serde_json::Error> for JobError {
    fn from(e: serde_json::Error) -> Self {
        JobError::Payload(e.to_string())
    }
}

impl From<sqlx::Error> for JobError {
    fn from(e: sqlx::Error) -> Self {
        JobError::Engine(anyhow::Error::new(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_error_maps_to_payload() {
        let err: JobError = serde_json::from_str::<i32>("not json").unwrap_err().into();
        assert!(matches!(err, JobError::Payload(_)));
        assert!(err.to_string().contains("payload error"));
    }

    #[test]
    fn sqlx_error_maps_to_engine() {
        let err: JobError = sqlx::Error::RowNotFound.into();
        assert!(matches!(err, JobError::Engine(_)));
        assert_eq!(err.to_string(), "queue engine error");
    }

    #[test]
    fn invalid_schedule_displays_name_and_reason() {
        let err = JobError::InvalidSchedule {
            name: "nightly".into(),
            reason: "bad cron".into(),
        };
        let s = err.to_string();
        assert!(s.contains("nightly") && s.contains("bad cron"));
    }
}
