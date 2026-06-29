//! Dead-letter handling (design D6). When a job exhausts its retries, sqlxmq
//! leaves the message in `mq_msgs` with `attempts = 0` and `attempt_at = NULL`
//! (never polled again) rather than deleting it. The worker's dead-letter sweep
//! identifies those, copies them into `jobs.dead_letter`, removes them from the
//! queue, and raises an alert — keeping ordered channels live (a poison job no
//! longer blocks its successors).
//!
//! The *decision* (is this message exhausted?) is pure and host-testable here;
//! the copy/delete/alert is engine glue.

/// A message that has exhausted its retries and must be dead-lettered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeadLetter {
    pub id: uuid::Uuid,
    pub name: String,
    pub channel_name: String,
    pub channel_args: String,
    pub attempts: i32,
    pub last_error: Option<String>,
}

/// Whether a queue message has exhausted its retry budget. sqlxmq decrements
/// `attempts` on each poll and sets `attempt_at = NULL` on the final attempt; a
/// message that has run out (`attempts <= 0`) and will never be attempted again
/// (`attempt_at IS NULL`) is dead.
pub fn is_exhausted(attempts: i32, attempt_at_is_null: bool) -> bool {
    attempts <= 0 && attempt_at_is_null
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhausted_when_no_attempts_and_no_next_time() {
        assert!(is_exhausted(0, true));
        assert!(is_exhausted(-1, true));
    }

    #[test]
    fn live_when_attempts_remain_or_scheduled() {
        assert!(!is_exhausted(2, false)); // still has tries
        assert!(!is_exhausted(2, true)); // tries left, just paused
        assert!(!is_exhausted(0, false)); // scheduled for one more run
    }

    #[test]
    fn dead_letter_record_roundtrips_fields() {
        let dl = DeadLetter {
            id: uuid::Uuid::nil(),
            name: "verification_email".into(),
            channel_name: "auth.email".into(),
            channel_args: String::new(),
            attempts: 0,
            last_error: Some("smtp down".into()),
        };
        assert_eq!(dl.name, "verification_email");
        assert_eq!(dl.last_error.as_deref(), Some("smtp down"));
    }
}
