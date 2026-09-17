// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Pure erasure rules (host-tested, no I/O; change: add-lingua-privacy-controls, design
//! D2): what a push may still store once the reader has erased their Lingua data. The
//! mark is the server time of the erasure, 0 when the user never erased.

const DAY_MS: i64 = 86_400_000;

/// Whether an op dated `ts` (already clamped to the server's receipt time) predates the
/// erasure and must be dropped — a device, or an extension built before the mark
/// existed, re-uploading what was erased.
pub fn predates_erasure(ts: i64, erased_at: i64) -> bool {
    erased_at > 0 && ts <= erased_at
}

/// Whether a daily stat for `day` (UTC day key, days since the Unix epoch) predates the
/// erasure's own UTC day and must be dropped. The erasure day itself is kept: a daily row
/// carries no time, and a wiped device starts that day's counts again from zero.
pub fn day_predates_erasure(day: i32, erased_at: i64) -> bool {
    erased_at > 0 && i64::from(day) < erased_at.div_euclid(DAY_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ops_at_or_before_the_mark_are_dropped() {
        assert!(predates_erasure(999, 1_000));
        assert!(predates_erasure(1_000, 1_000));
        assert!(!predates_erasure(1_001, 1_000));
    }

    #[test]
    fn nothing_is_dropped_without_an_erasure() {
        assert!(!predates_erasure(0, 0));
        assert!(!predates_erasure(-5, 0));
        assert!(!day_predates_erasure(0, 0));
    }

    #[test]
    fn only_days_before_the_erasure_day_are_dropped() {
        let erased_at = 20_000 * DAY_MS + 3_600_000; // 01:00 UTC on day 20000
        assert!(day_predates_erasure(19_999, erased_at));
        assert!(!day_predates_erasure(20_000, erased_at));
        assert!(!day_predates_erasure(20_001, erased_at));
    }
}
