//! Attempt tracking vocabulary (change: add-admin-jobs-console, design D1).
//!
//! sqlxmq deletes a job when it completes and writes nothing when a handler fails, so
//! the queue alone cannot say how much work was done, nor tell a running job from one
//! waiting for its retry. The worker therefore records every attempt around its
//! handler in `jobs.job_attempts`. This module holds the pure part — what an attempt's
//! outcome is, what starting one can find, how long the history is kept; the SQL that
//! writes it is engine glue ([`crate::engine::tracked`]).

/// How long attempt history and cancellation records are kept. The console's period
/// window cannot start further back, and the dead-letter sweep prunes past it.
pub const HISTORY_RETENTION_DAYS: i64 = 90;

/// How a finished attempt ended, as the worker reports it. `running` and `abandoned`
/// also exist in the table, but they are written by the start of an attempt and by the
/// sweep, never as a handler's result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptOutcome {
    Succeeded,
    Failed,
}

impl AttemptOutcome {
    /// The outcome a handler's result stands for.
    pub fn of<T, E>(result: &Result<T, E>) -> Self {
        if result.is_ok() {
            Self::Succeeded
        } else {
            Self::Failed
        }
    }

    /// The `jobs.job_attempts.outcome` value.
    pub fn as_db(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

/// What the worker found when it went to start an attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptStart {
    /// The attempt is recorded as running; the id of its history row.
    Started(i64),
    /// The message left the queue between the claim and the start — an operator
    /// cancelled it. The handler must not run.
    Gone,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_follows_the_handler_result() {
        let ok: Result<(), &str> = Ok(());
        let err: Result<(), &str> = Err("smtp down");
        assert_eq!(AttemptOutcome::of(&ok), AttemptOutcome::Succeeded);
        assert_eq!(AttemptOutcome::of(&err), AttemptOutcome::Failed);
    }

    #[test]
    fn outcome_words_match_the_table_check() {
        // `jobs.job_attempts.outcome` CHECK: running | succeeded | failed | abandoned.
        assert_eq!(AttemptOutcome::Succeeded.as_db(), "succeeded");
        assert_eq!(AttemptOutcome::Failed.as_db(), "failed");
    }

    #[test]
    fn retention_matches_the_documented_window() {
        assert_eq!(HISTORY_RETENTION_DAYS, 90);
    }

    #[test]
    fn a_gone_start_is_not_a_started_attempt() {
        assert_ne!(AttemptStart::Gone, AttemptStart::Started(1));
    }
}
