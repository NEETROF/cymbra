// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Pure, host-testable leaderboard logic (change: add-play-leaderboards): derive
//! the per-mode best candidates from an ingested session-result record (with the
//! basic integrity checks, design D5), the **monotonic** better-than comparison
//! that keeps a personal best from ever regressing, and the ranking order + own-
//! rank computation. No I/O and no gRPC — so the money logic (idempotency,
//! tie-break, gating math) is fully unit-tested.

use std::cmp::Ordering;

use crate::leaderboard::{BestCandidate, Mode, StoredBest};

/// Tie-break sentinel for a candidate whose mode had a sub-score but no timing
/// hits (e.g. every free onset missed): larger than any real metric, so it sorts
/// last on a tie without special-casing the read. Well under `f32::MAX`.
pub const NO_TIMING_TIEBREAK: f32 = 1.0e9;

/// Max plausible mean free-run tempo offset (ms); a larger magnitude is a bad/
/// forged result and disqualifies the tempo candidate (design D5).
const MAX_TEMPO_OFFSET_MS: f64 = 5_000.0;

/// Max plausible mean Wait-Mode reaction time (ms); beyond it the reaction
/// candidate is disqualified.
const MAX_REACTION_MS: f64 = 10_000.0;

/// The per-mode candidates derived from one session, split into the board-eligible
/// [`BestCandidate`]s and the human-readable reasons a mode was rejected (for the
/// module to log — the session itself is still stored by #5, just kept off boards).
#[derive(Debug, Default, PartialEq)]
pub struct Candidates {
    pub accepted: Vec<BestCandidate>,
    pub rejected: Vec<String>,
}

/// Derive the board candidates from a session-result record (`session_result_json`
/// — performance-scoring's serializable record) achieved at `achieved_at_ms`.
///
/// Reads the per-mode sub-scores (`freeSyncPct`/`waitSyncPct`), per-mode onset
/// counts, and timing metrics, applying the basic integrity checks (design D5):
/// a sub-score must be in `[0, 100]`, its mode must have had onsets, and the
/// timing metric (when present) must be within plausible bounds. A mode that
/// fails is excluded from the boards and its reason recorded in `rejected`.
///
/// `Err` only for a **malformed** record (not valid JSON / not an object) — the
/// whole session then contributes nothing to any board.
pub fn candidates_from_result(json: &str, achieved_at_ms: i64) -> Result<Candidates, String> {
    // An empty detail (client stored nothing) simply yields no candidates.
    if json.trim().is_empty() {
        return Ok(Candidates::default());
    }
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("malformed session_result: {e}"))?;
    let obj = v
        .as_object()
        .ok_or_else(|| "session_result is not a JSON object".to_string())?;

    let f64_of = |key: &str| obj.get(key).and_then(serde_json::Value::as_f64);
    let i64_of = |key: &str| obj.get(key).and_then(serde_json::Value::as_i64);

    let mut out = Candidates::default();

    // Tempo board — from the free-run sub-score.
    if let Some(sub) = f64_of("freeSyncPct") {
        let onsets = i64_of("freeOnsetCount").unwrap_or(0);
        let offset = f64_of("avgFreeOffsetMs"); // signed; |mean| is the tie-break
        match check_candidate(sub, onsets, offset.map(f64::abs), MAX_TEMPO_OFFSET_MS) {
            Ok(tiebreak) => out.accepted.push(BestCandidate {
                mode: Mode::Tempo,
                subscore: sub as f32,
                tiebreak_metric: tiebreak,
                achieved_at_ms,
            }),
            Err(why) => out.rejected.push(format!("tempo: {why}")),
        }
    }

    // Reaction board — from the Wait-Mode sub-score.
    if let Some(sub) = f64_of("waitSyncPct") {
        let onsets = i64_of("waitOnsetCount").unwrap_or(0);
        let reaction = f64_of("avgReactionMs"); // non-negative; faster = smaller
        match check_candidate(sub, onsets, reaction, MAX_REACTION_MS) {
            Ok(tiebreak) => out.accepted.push(BestCandidate {
                mode: Mode::Reaction,
                subscore: sub as f32,
                tiebreak_metric: tiebreak,
                achieved_at_ms,
            }),
            Err(why) => out.rejected.push(format!("reaction: {why}")),
        }
    }

    Ok(out)
}

/// Validate one mode's sub-score/onsets/timing and resolve its tie-break metric
/// (normalised so smaller is better). `metric` is already non-negative (an
/// absolute offset or a reaction time); `None` ⇒ the mode had a sub-score but no
/// timing hits, which is allowed and yields the [`NO_TIMING_TIEBREAK`] sentinel.
fn check_candidate(
    subscore: f64,
    onsets: i64,
    metric: Option<f64>,
    max_metric: f64,
) -> Result<f32, String> {
    if !subscore.is_finite() || !(0.0..=100.0).contains(&subscore) {
        return Err(format!("sub-score {subscore} out of range [0,100]"));
    }
    if onsets <= 0 {
        return Err(format!("sub-score present but onset count is {onsets}"));
    }
    match metric {
        None => Ok(NO_TIMING_TIEBREAK),
        Some(m) if m.is_finite() && (0.0..=max_metric).contains(&m) => Ok(m as f32),
        Some(m) => Err(format!("timing metric {m} outside plausible bounds")),
    }
}

/// Whether a new result `(new_sub, new_tie, new_at)` is **strictly better** than
/// the stored best `(cur_sub, cur_tie, cur_at)` under the ranking order — the
/// predicate that makes the personal-best upsert monotonic (never regresses) and
/// idempotent (an equal replay is not "better", so it is a no-op).
///
/// Better = higher sub-score; on a tie, a smaller tie-break metric; on a further
/// tie, an earlier `achieved_at`.
#[allow(clippy::too_many_arguments)]
pub fn is_better(
    new_sub: f32,
    new_tie: f32,
    new_at: i64,
    cur_sub: f32,
    cur_tie: f32,
    cur_at: i64,
) -> bool {
    rank_cmp(new_sub, new_tie, new_at, cur_sub, cur_tie, cur_at) == Ordering::Less
}

/// Ranking comparator between two bests: the one that sorts **`Less` ranks
/// higher** (rank 1 first). Higher sub-score first; ties by smaller tie-break
/// metric; then by earlier `achieved_at`.
#[allow(clippy::too_many_arguments)]
pub fn rank_cmp(a_sub: f32, a_tie: f32, a_at: i64, b_sub: f32, b_tie: f32, b_at: i64) -> Ordering {
    // Higher sub-score ranks first ⇒ compare b vs a so a bigger score is `Less`.
    b_sub
        .partial_cmp(&a_sub)
        .unwrap_or(Ordering::Equal)
        .then_with(|| a_tie.partial_cmp(&b_tie).unwrap_or(Ordering::Equal))
        .then_with(|| a_at.cmp(&b_at))
}

/// The 1-based rank of `own` among the already-ranking-ordered public entries
/// `public_ranked`: one plus the number of public entries that rank strictly
/// higher. Works whether or not `own` is itself listed (a private caller still
/// gets "your position among the public players").
pub fn own_rank(public_ranked: &[StoredBest], own: &StoredBest) -> i32 {
    let ahead = public_ranked
        .iter()
        .filter(|e| {
            rank_cmp(
                e.subscore,
                e.tiebreak_metric,
                e.achieved_at_ms,
                own.subscore,
                own.tiebreak_metric,
                own.achieved_at_ms,
            ) == Ordering::Less
        })
        .count();
    (ahead as i32) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stored(user: &str, sub: f32, tie: f32, at: i64) -> StoredBest {
        StoredBest {
            user_id: user.into(),
            subscore: sub,
            tiebreak_metric: tie,
            achieved_at_ms: at,
        }
    }

    #[test]
    fn parses_both_modes_from_a_mixed_run() {
        let json = r#"{
            "freeSyncPct": 82.0, "freeOnsetCount": 10, "avgFreeOffsetMs": -12.5,
            "waitSyncPct": 91.0, "waitOnsetCount": 8,  "avgReactionMs": 140.0
        }"#;
        let c = candidates_from_result(json, 1000).unwrap();
        assert_eq!(c.rejected, Vec::<String>::new());
        assert_eq!(c.accepted.len(), 2);
        let tempo = c.accepted.iter().find(|c| c.mode == Mode::Tempo).unwrap();
        assert_eq!(tempo.subscore, 82.0);
        // Tie-break is the ABSOLUTE mean offset (12.5), not the signed value.
        assert_eq!(tempo.tiebreak_metric, 12.5);
        let react = c
            .accepted
            .iter()
            .find(|c| c.mode == Mode::Reaction)
            .unwrap();
        assert_eq!(react.tiebreak_metric, 140.0);
    }

    #[test]
    fn pure_run_yields_one_candidate() {
        let json = r#"{"freeSyncPct": 70.0, "freeOnsetCount": 5, "avgFreeOffsetMs": 8.0}"#;
        let c = candidates_from_result(json, 1).unwrap();
        assert_eq!(c.accepted.len(), 1);
        assert_eq!(c.accepted[0].mode, Mode::Tempo);
    }

    #[test]
    fn missing_timing_uses_the_sentinel() {
        // A sub-score with no timing hits (all missed) is allowed but sorts last.
        let json = r#"{"freeSyncPct": 10.0, "freeOnsetCount": 4}"#;
        let c = candidates_from_result(json, 1).unwrap();
        assert_eq!(c.accepted[0].tiebreak_metric, NO_TIMING_TIEBREAK);
    }

    #[test]
    fn out_of_range_subscore_is_rejected() {
        let json = r#"{"freeSyncPct": 150.0, "freeOnsetCount": 4, "avgFreeOffsetMs": 5.0}"#;
        let c = candidates_from_result(json, 1).unwrap();
        assert!(c.accepted.is_empty());
        assert_eq!(c.rejected.len(), 1);
        assert!(c.rejected[0].contains("out of range"));
    }

    #[test]
    fn subscore_without_onsets_is_rejected() {
        let json = r#"{"waitSyncPct": 88.0, "waitOnsetCount": 0, "avgReactionMs": 100.0}"#;
        let c = candidates_from_result(json, 1).unwrap();
        assert!(c.accepted.is_empty());
        assert!(c.rejected[0].contains("onset count"));
    }

    #[test]
    fn implausible_timing_is_rejected() {
        let json = r#"{"waitSyncPct": 88.0, "waitOnsetCount": 3, "avgReactionMs": 99999.0}"#;
        let c = candidates_from_result(json, 1).unwrap();
        assert!(c.accepted.is_empty());
        assert!(c.rejected[0].contains("plausible bounds"));
    }

    #[test]
    fn malformed_json_errors() {
        assert!(candidates_from_result("not json", 1).is_err());
    }

    #[test]
    fn empty_detail_yields_no_candidates() {
        assert_eq!(
            candidates_from_result("", 1).unwrap(),
            Candidates::default()
        );
    }

    #[test]
    fn better_by_subscore_then_tiebreak_then_recency() {
        // Higher sub-score is better.
        assert!(is_better(90.0, 50.0, 100, 80.0, 10.0, 100));
        // Equal sub-score: smaller tie-break wins.
        assert!(is_better(80.0, 5.0, 100, 80.0, 10.0, 100));
        // Equal sub-score + tie-break: earlier achieved wins.
        assert!(is_better(80.0, 10.0, 50, 80.0, 10.0, 100));
        // An identical replay is NOT better (idempotent no-op).
        assert!(!is_better(80.0, 10.0, 100, 80.0, 10.0, 100));
        // A worse sub-score is not better.
        assert!(!is_better(70.0, 1.0, 1, 80.0, 10.0, 100));
    }

    #[test]
    fn own_rank_counts_public_players_ahead() {
        let ranked = vec![
            stored("a", 95.0, 10.0, 1),
            stored("b", 90.0, 5.0, 1),
            stored("c", 80.0, 5.0, 1),
        ];
        // A private caller with an 88 slots between b and c → rank 3.
        let own = stored("me", 88.0, 1.0, 1);
        assert_eq!(own_rank(&ranked, &own), 3);
        // Top score → rank 1.
        assert_eq!(own_rank(&ranked, &stored("me", 99.0, 1.0, 1)), 1);
        // Below everyone → last + 1.
        assert_eq!(own_rank(&ranked, &stored("me", 10.0, 1.0, 1)), 4);
    }
}
