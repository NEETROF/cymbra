// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Pure sync-resolution logic (host-tested, no I/O): last-write-wins with a device
//! tie-break, and client-clock defence. Both the in-memory fake and the Postgres
//! adapter apply the same rule, so convergence is decided here, not in SQL.

/// Clamp a client timestamp against the server's receipt time: a future timestamp
/// (skewed client clock) is pulled back to `now`, so no client can pin a lemma
/// "in the future" and win forever. Past timestamps are kept as-is.
pub fn clamp_ts(client_ts: i64, now: i64) -> i64 {
    client_ts.min(now)
}

/// Whether an incoming op should overwrite the stored one under last-write-wins:
/// the later timestamp wins; on a tie, the lexicographically greater `device_id`
/// wins (deterministic, so every device converges to the same result).
pub fn wins(
    existing_ts: i64,
    existing_device: &str,
    incoming_ts: i64,
    incoming_device: &str,
) -> bool {
    match incoming_ts.cmp(&existing_ts) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => incoming_device > existing_device,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn future_timestamps_are_clamped_to_now() {
        assert_eq!(clamp_ts(2000, 1000), 1000); // future -> now
        assert_eq!(clamp_ts(500, 1000), 500); // past kept
        assert_eq!(clamp_ts(1000, 1000), 1000);
    }

    #[test]
    fn later_timestamp_wins() {
        assert!(wins(100, "a", 200, "a"));
        assert!(!wins(200, "a", 100, "a"));
    }

    #[test]
    fn equal_timestamps_break_by_device_id() {
        assert!(wins(100, "device-a", 100, "device-b")); // b > a
        assert!(!wins(100, "device-b", 100, "device-a"));
        assert!(!wins(100, "device-a", 100, "device-a")); // identical => no change
    }
}
