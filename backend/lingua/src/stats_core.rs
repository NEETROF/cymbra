// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0

//! Pure stats consolidation (host-tested): sum per-device daily rows into one value
//! per (day, language). Keeping it here means the "SUM across devices on read"
//! contract is tested without a database.

use std::collections::BTreeMap;

/// A device's aggregate for one UTC day + language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DailyStat {
    pub day: i32,
    pub language: String,
    pub device_id: String,
    pub exposures: u32,
    pub words_learned: u32,
    pub reviews_done: u32,
}

/// A consolidated aggregate, summed across every device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsolidatedStat {
    pub day: i32,
    pub language: String,
    pub exposures: u32,
    pub words_learned: u32,
    pub reviews_done: u32,
}

/// Sum per-device rows into one row per (day, language), in (day, language) order.
/// Exactly one value per day per measure, as the spec requires.
pub fn consolidate(rows: &[DailyStat]) -> Vec<ConsolidatedStat> {
    let mut by_key: BTreeMap<(i32, String), ConsolidatedStat> = BTreeMap::new();
    for row in rows {
        let entry = by_key
            .entry((row.day, row.language.clone()))
            .or_insert(ConsolidatedStat {
                day: row.day,
                language: row.language.clone(),
                exposures: 0,
                words_learned: 0,
                reviews_done: 0,
            });
        entry.exposures += row.exposures;
        entry.words_learned += row.words_learned;
        entry.reviews_done += row.reviews_done;
    }
    by_key.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stat(day: i32, device: &str, reviews: u32) -> DailyStat {
        DailyStat {
            day,
            language: "en".into(),
            device_id: device.into(),
            exposures: 0,
            words_learned: 0,
            reviews_done: reviews,
        }
    }

    #[test]
    fn sums_two_devices_on_the_same_day() {
        let rows = vec![stat(20_000, "mac", 20), stat(20_000, "iphone", 10)];
        let out = consolidate(&rows);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].reviews_done, 30);
        assert_eq!(out[0].day, 20_000);
    }

    #[test]
    fn keeps_days_and_languages_separate_and_ordered() {
        let rows = vec![
            stat(20_001, "mac", 5),
            stat(20_000, "mac", 3),
            DailyStat {
                language: "fr".into(),
                ..stat(20_000, "mac", 7)
            },
        ];
        let out = consolidate(&rows);
        assert_eq!(out.len(), 3);
        // Ordered by (day, language): (20000,en), (20000,fr), (20001,en).
        assert_eq!((out[0].day, out[0].language.as_str()), (20_000, "en"));
        assert_eq!((out[1].day, out[1].language.as_str()), (20_000, "fr"));
        assert_eq!((out[2].day, out[2].language.as_str()), (20_001, "en"));
    }

    #[test]
    fn empty_input_is_empty() {
        assert!(consolidate(&[]).is_empty());
    }
}
