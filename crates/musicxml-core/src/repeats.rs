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

//! The playback-order **unroll**: from the repeat structure engraved on the
//! written measures ([`crate::model::RepeatMarks`]), the sequence of
//! written-measure passes a performer would play. Computed once here and
//! consumed by every derivation (app timeline, browser preview, server audio
//! render) — never re-implemented per consumer.
//!
//! Resolution follows standard performance rules: forward/backward repeats
//! (honouring `times`), volta brackets selected by pass number (passes beyond
//! the listed brackets replay the last), and at most one D.C./D.S. jump —
//! with repeats not re-taken after the jump unless `<sound
//! forward-repeat="yes">` says otherwise, ending at Fine or jumping to the
//! coda when marked. The unroll is **capped**; on any inconsistency —
//! unmatched volta, missing jump target, cap exceeded — it falls back to the
//! written order one-to-one, the pre-repeat behavior.

use crate::model::{NotationMeasure, PlayedMeasure};

/// Total played measures may not exceed this multiple of the written count…
const MAX_MULTIPLIER: usize = 8;
/// …nor this absolute ceiling.
const MAX_TOTAL: usize = 4096;

/// The playback order for [`measures`]: written order one-to-one when the
/// piece has no repeats or its structure cannot be resolved safely.
pub fn play_order(measures: &[NotationMeasure]) -> Vec<PlayedMeasure> {
    resolve(measures).unwrap_or_else(|| fallback(measures))
}

fn fallback(measures: &[NotationMeasure]) -> Vec<PlayedMeasure> {
    (0..measures.len())
        .map(|i| PlayedMeasure {
            written_index: i as u32,
            pass: 1,
        })
        .collect()
}

/// Volta coverage: for each measure, the bracket numbers covering it (a
/// bracket spans from its `start` to the measure carrying `stop`/
/// `discontinue`, inclusive), plus the highest number of the bracket *group*
/// (adjacent brackets — "1." then "2.") for the passes-beyond-last rule.
/// `None` on overlapping or unterminated brackets.
type VoltaCoverage = (Vec<Option<Vec<u32>>>, Vec<u32>);

fn volta_coverage(measures: &[NotationMeasure]) -> Option<VoltaCoverage> {
    let n = measures.len();
    let mut coverage: Vec<Option<Vec<u32>>> = vec![None; n];
    let mut open: Option<Vec<u32>> = None;
    for (i, m) in measures.iter().enumerate() {
        let r = &m.repeats;
        if !r.ending_start.is_empty() {
            if open.is_some() {
                return None; // overlapping brackets
            }
            open = Some(r.ending_start.clone());
        }
        if let Some(nums) = &open {
            coverage[i] = Some(nums.clone());
        }
        if r.ending_stop || r.ending_discontinue {
            open.take()?; // stop without a start
        }
    }
    if open.is_some() {
        return None; // unterminated bracket
    }
    // Group max: maximal runs of consecutive covered measures share it.
    let mut group_max = vec![0u32; n];
    let mut i = 0;
    while i < n {
        if coverage[i].is_none() {
            i += 1;
            continue;
        }
        let start = i;
        let mut max = 0u32;
        while i < n && coverage[i].is_some() {
            for v in coverage[i].as_ref().unwrap() {
                max = max.max(*v);
            }
            i += 1;
        }
        for g in &mut group_max[start..i] {
            *g = max;
        }
    }
    Some((coverage, group_max))
}

fn resolve(measures: &[NotationMeasure]) -> Option<Vec<PlayedMeasure>> {
    let n = measures.len();
    if n == 0 {
        return None;
    }
    let (coverage, group_max) = volta_coverage(measures)?;
    let has_jump = measures
        .iter()
        .any(|m| m.repeats.sound_dacapo || m.repeats.sound_dalsegno);
    let segno_at = measures.iter().position(|m| m.repeats.segno);
    let coda_at = measures.iter().position(|m| m.repeats.coda);
    if has_jump && measures.iter().any(|m| m.repeats.sound_dalsegno) && segno_at.is_none() {
        return None; // D.S. without a segno
    }

    let cap = (n * MAX_MULTIPLIER).min(MAX_TOTAL);
    let mut out: Vec<PlayedMeasure> = Vec::new();
    let mut i = 0usize;
    let mut pass: u32 = 1;
    let mut anchor: usize = 0; // where a backward repeat returns to
    let mut jumped = false; // the one D.C./D.S. has been taken
    let mut after_jump = false;
    let mut in_coda = false;
    let mut take_repeats_after_jump = false;
    let mut prev_in_group = false;
    let mut moved_forward = true;
    let mut steps = 0usize;

    while i < n {
        steps += 1;
        if steps > cap * 2 + 16 {
            return None; // structural loop that emits nothing
        }
        let r = &measures[i].repeats;
        let in_group = coverage[i].is_some();
        // Walking forward past the end of a bracket group resolves the whole
        // repeated section: the pass counter and the return anchor reset.
        if moved_forward && prev_in_group && !in_group {
            pass = 1;
            anchor = i;
        }
        prev_in_group = in_group;
        moved_forward = true;

        if r.forward {
            anchor = i;
        }

        // Volta selection by pass; beyond the listed brackets (and after a
        // jump with repeats not re-taken) the last bracket plays.
        if let Some(nums) = &coverage[i] {
            let gmax = group_max[i].max(1);
            let effective = if after_jump && !take_repeats_after_jump {
                gmax
            } else {
                pass.min(gmax)
            };
            if !nums.contains(&effective) {
                i += 1;
                continue; // this ending is not played on this pass
            }
        }

        out.push(PlayedMeasure {
            written_index: i as u32,
            pass,
        });
        if out.len() > cap {
            return None;
        }

        // Post-jump terminations: al Fine ends at Fine, al Coda leaps to it.
        if after_jump && r.sound_fine {
            break;
        }
        if after_jump && !in_coda && r.sound_tocoda {
            match coda_at {
                Some(c) if c > i => {
                    i = c;
                    in_coda = true;
                    pass = 1;
                    prev_in_group = false;
                    moved_forward = false;
                    continue;
                }
                _ => return None, // "to Coda" with no coda ahead
            }
        }

        // Backward repeat: go back while passes remain (not re-taken after a
        // jump unless the score says so).
        if r.backward_times > 0 && (!after_jump || take_repeats_after_jump) {
            if pass < r.backward_times {
                if anchor > i {
                    return None;
                }
                i = anchor;
                pass += 1;
                moved_forward = false;
                continue;
            }
            // Section resolved: a later anchor-less backward returns here.
            pass = 1;
            anchor = i + 1;
        }

        // The one D.C./D.S. jump.
        if !jumped && (r.sound_dacapo || r.sound_dalsegno) {
            jumped = true;
            after_jump = true;
            take_repeats_after_jump = r.sound_forward_repeat;
            pass = 1;
            prev_in_group = false;
            moved_forward = false;
            if r.sound_dalsegno {
                i = segno_at?; // checked above, still guard
            } else {
                i = 0;
                anchor = 0;
            }
            continue;
        }

        i += 1;
    }

    if out.is_empty() {
        return None; // e.g. voltas that skip every measure
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn order(xml: &str) -> Vec<u32> {
        parse(xml.as_bytes())
            .unwrap()
            .play_order
            .iter()
            .map(|p| p.written_index)
            .collect()
    }

    /// A minimal piece: [header] + per-measure bodies supplied by the caller.
    fn piece(measure_bodies: &[&str]) -> String {
        let mut s = String::from(
            r#"<?xml version="1.0"?><score-partwise version="3.1">
  <part-list><score-part id="P1"/></part-list><part id="P1">"#,
        );
        for (i, body) in measure_bodies.iter().enumerate() {
            s.push_str(&format!(
                "<measure number=\"{}\">{}{}</measure>",
                i + 1,
                if i == 0 {
                    "<attributes><divisions>4</divisions><time><beats>4</beats><beat-type>4</beat-type></time></attributes>"
                } else {
                    ""
                },
                body
            ));
        }
        s.push_str("</part></score-partwise>");
        s
    }

    const NOTE: &str = r#"<note><pitch><step>C</step><octave>4</octave></pitch><duration>16</duration><voice>1</voice><type>whole</type></note>"#;
    const FWD: &str = r#"<barline location="left"><repeat direction="forward"/></barline>"#;
    const BWD: &str = r#"<barline location="right"><repeat direction="backward"/></barline>"#;

    #[test]
    fn no_repeats_is_written_order() {
        let xml = piece(&[NOTE, NOTE, NOTE]);
        assert_eq!(order(&xml), vec![0, 1, 2]);
        let doc = parse(xml.as_bytes()).unwrap();
        assert!(doc.play_order.iter().all(|p| p.pass == 1));
    }

    #[test]
    fn simple_repeat_plays_twice() {
        let m1 = format!("{FWD}{NOTE}");
        let m2 = format!("{NOTE}{BWD}");
        let xml = piece(&[&m1, &m2, NOTE]);
        assert_eq!(order(&xml), vec![0, 1, 0, 1, 2]);
        let doc = parse(xml.as_bytes()).unwrap();
        assert_eq!(
            doc.play_order.iter().map(|p| p.pass).collect::<Vec<_>>(),
            vec![1, 1, 2, 2, 1]
        );
    }

    #[test]
    fn times_attribute_plays_thrice() {
        let m2 = format!(
            r#"{NOTE}<barline location="right"><repeat direction="backward" times="3"/></barline>"#
        );
        let m1 = format!("{FWD}{NOTE}");
        let xml = piece(&[&m1, &m2, NOTE]);
        assert_eq!(order(&xml), vec![0, 1, 0, 1, 0, 1, 2]);
    }

    #[test]
    fn backward_without_forward_returns_to_start() {
        let m2 = format!("{NOTE}{BWD}");
        let xml = piece(&[NOTE, &m2, NOTE]);
        assert_eq!(order(&xml), vec![0, 1, 0, 1, 2]);
    }

    #[test]
    fn voltas_select_per_pass() {
        // m0 (repeated) | m1 volta 1 + backward | m2 volta 2 | m3
        let m0 = format!("{FWD}{NOTE}");
        let v1 = format!(
            r#"<barline location="left"><ending number="1" type="start"/></barline>{NOTE}<barline location="right"><ending number="1" type="stop"/><repeat direction="backward"/></barline>"#
        );
        let v2 = format!(
            r#"<barline location="left"><ending number="2" type="start"/></barline>{NOTE}<barline location="right"><ending number="2" type="discontinue"/></barline>"#
        );
        let xml = piece(&[&m0, &v1, &v2, NOTE]);
        assert_eq!(order(&xml), vec![0, 1, 0, 2, 3]);
    }

    #[test]
    fn volta_number_lists_and_pass_beyond_last() {
        // Played three times; "1,2" bracket then "3" bracket.
        let m0 =
            format!(r#"<barline location="left"><repeat direction="forward"/></barline>{NOTE}"#);
        let v12 = format!(
            r#"<barline location="left"><ending number="1,2" type="start"/></barline>{NOTE}<barline location="right"><ending number="1,2" type="stop"/><repeat direction="backward" times="3"/></barline>"#
        );
        let v3 = format!(
            r#"<barline location="left"><ending number="3" type="start"/></barline>{NOTE}<barline location="right"><ending number="3" type="discontinue"/></barline>"#
        );
        let xml = piece(&[&m0, &v12, &v3, NOTE]);
        assert_eq!(order(&xml), vec![0, 1, 0, 1, 0, 2, 3]);
    }

    #[test]
    fn da_capo_al_fine() {
        // m0 | m1 Fine | m2 D.C. al Fine → 0,1,2,0,1(stop)
        let fine = format!(r#"{NOTE}<direction><sound fine="yes"/></direction>"#);
        let dc = format!(r#"{NOTE}<direction><sound dacapo="yes"/></direction>"#);
        let xml = piece(&[NOTE, &fine, &dc]);
        assert_eq!(order(&xml), vec![0, 1, 2, 0, 1]);
    }

    #[test]
    fn dal_segno_al_coda() {
        // m0 | m1 segno | m2 to-coda | m3 D.S. | m4 coda | m5
        let segno =
            format!(r#"<direction><direction-type><segno/></direction-type></direction>{NOTE}"#);
        let tocoda = format!(r#"{NOTE}<direction><sound tocoda="coda"/></direction>"#);
        let ds = format!(r#"{NOTE}<direction><sound dalsegno="segno"/></direction>"#);
        let coda =
            format!(r#"<direction><direction-type><coda/></direction-type></direction>{NOTE}"#);
        let xml = piece(&[NOTE, &segno, &tocoda, &ds, &coda, NOTE]);
        // 0,1,2,3 → jump to segno (1) → 2 (to-coda fires) → coda (4), 5.
        assert_eq!(order(&xml), vec![0, 1, 2, 3, 1, 2, 4, 5]);
    }

    #[test]
    fn repeats_not_retaken_after_da_capo() {
        // m0‖: m1 :‖ m2 D.C. — replay skips the repeat.
        let m0 = format!("{FWD}{NOTE}");
        let m1 = format!("{NOTE}{BWD}");
        let dc = format!(r#"{NOTE}<direction><sound dacapo="yes" fine="yes"/></direction>"#);
        // Put fine on the D.C. measure itself so the replay ends there.
        let xml = piece(&[&m0, &m1, &dc]);
        assert_eq!(order(&xml), vec![0, 1, 0, 1, 2, 0, 1, 2]);
    }

    #[test]
    fn malformed_volta_falls_back_to_written_order() {
        // A stop with no start.
        let bad = format!(
            r#"{NOTE}<barline location="right"><ending number="1" type="stop"/></barline>"#
        );
        let xml = piece(&[NOTE, &bad, NOTE]);
        assert_eq!(order(&xml), vec![0, 1, 2]);
    }

    #[test]
    fn dal_segno_without_segno_falls_back() {
        let ds = format!(r#"{NOTE}<direction><sound dalsegno="segno"/></direction>"#);
        let xml = piece(&[NOTE, &ds, NOTE]);
        assert_eq!(order(&xml), vec![0, 1, 2]);
    }

    #[test]
    fn unroll_is_capped() {
        // times="4000" on a 3-measure piece exceeds 8× the written count.
        let m1 = format!("{FWD}{NOTE}");
        let m2 = format!(
            r#"{NOTE}<barline location="right"><repeat direction="backward" times="4000"/></barline>"#
        );
        let xml = piece(&[&m1, &m2, NOTE]);
        assert_eq!(order(&xml), vec![0, 1, 2]); // fallback, not 8000 slots
    }

    #[test]
    fn fuzzed_repeat_marks_always_terminate_in_bounds() {
        // Deterministic LCG-driven soups of repeat marks: the unroll must
        // terminate, stay within the cap, and only reference real measures.
        let mut seed: u64 = 0x00C0FFEE;
        let mut rand = move || {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) as u32
        };
        for _ in 0..200 {
            let n = 1 + (rand() % 6) as usize;
            let bodies: Vec<String> = (0..n)
                .map(|_| {
                    let mut b = String::new();
                    if rand() % 3 == 0 {
                        b.push_str(FWD);
                    }
                    if rand() % 4 == 0 {
                        b.push_str(&format!(
                            r#"<barline location="left"><ending number="{}" type="start"/></barline>"#,
                            1 + rand() % 3
                        ));
                    }
                    b.push_str(NOTE);
                    if rand() % 4 == 0 {
                        b.push_str(&format!(
                            r#"<barline location="right"><ending number="1" type="{}"/></barline>"#,
                            if rand() % 2 == 0 { "stop" } else { "discontinue" }
                        ));
                    }
                    if rand() % 3 == 0 {
                        b.push_str(&format!(
                            r#"<barline location="right"><repeat direction="backward" times="{}"/></barline>"#,
                            1 + rand() % 5
                        ));
                    }
                    if rand() % 5 == 0 {
                        b.push_str(r#"<direction><sound dacapo="yes"/></direction>"#);
                    }
                    if rand() % 6 == 0 {
                        b.push_str(r#"<direction><sound fine="yes"/></direction>"#);
                    }
                    b
                })
                .collect();
            let refs: Vec<&str> = bodies.iter().map(|s| s.as_str()).collect();
            let doc = parse(piece(&refs).as_bytes()).unwrap();
            assert!(!doc.play_order.is_empty());
            assert!(doc.play_order.len() <= (n * MAX_MULTIPLIER).min(MAX_TOTAL).max(n));
            assert!(
                doc.play_order
                    .iter()
                    .all(|p| (p.written_index as usize) < n)
            );
        }
    }
}
