## 1. Stroke identity (the matcher's foundation)

- [x] 1.1 `apps/music/lib/state/drum_kit.dart`: expose the named-piece table as the matcher's equivalence source — a pure `samePiece(writtenGm, incomingGm)` (or piece-id resolver) keyed to the **static table**, not to the score-derived lane layout, so an incoming 40 matches a written 38 in a score that never contains a 40. Do NOT duplicate the table
- [x] 1.2 Unit tests over the groups: snare {37, 38, 40} inter-satisfies; hi-hat {42, 46} inter-satisfies (binding — the verdict cap is 2.3); ride {51, 53, 59} inter-satisfies; kick {35, 36} inter-satisfies; crash 49 vs china 52 do NOT; tom vs tom do NOT; terminal-bucket numbers match only themselves
- [x] 1.3 One identity, every consumer: the Wait gate's required set, the scorer's binding, extra-stroke detection and the pad feedback all call the same function — add a test asserting the gate releases exactly when the scorer binds (the drift the one-identity requirement forbids)

## 2. Percussion scoring core (Flutter)

- [x] 2.1 `performance_scoring_core.dart`: a percussion blend — timing and correctness only, weights renormalized to preserve the keyboard ratio (0.5 : 0.3 → 0.625 : 0.375); keyboard constants byte-for-byte untouched. A do-nothing percussion run scores 0, not the sustain weight
- [x] 2.2 `performance_scoring.dart`: judge percussion attacks through the stroke identity (1.1) in place of pitch equality — same windows, same verdict scale, both mode metrics (free signed offset, Wait reaction); velocity ignored
- [x] 2.3 Open/closed hi-hat shading: a wrong-articulation stroke binds and releases the gate, verdict capped below `perfect`; a right-articulation stroke judged on timing alone. Tests for both, including the no-hi-hat-controller case (closed against written open completes the run)
- [x] 2.4 No sustain for percussion: no ratio derived, no penalty possible; the session-result record's sustain aggregate and per-note sustain fields **absent, not zero**, and round-tripping serialization preserves the absence
- [x] 2.5 Extra-stroke detection: a stroke of a piece no open onset requires lowers correctness and never freezes playback; a stroke of a required piece within the window credits its onset
- [x] 2.6 Run activation: a full percussion run in the cascade arms the scorer; a selective (measure-range) percussion run does NOT (unscored by construction — assert the scorer is never armed, same seam as keyboard); mode stamping/classification (`free`/`wait`/`mixed`) unchanged over the percussion path
- [x] 2.7 Hands/feet scoping: the judged set and the recorded selection follow the hands/feet classification; a feet-only run judges only foot events

## 3. Wait Mode for percussion (Flutter)

- [x] 3.1 Offer Wait Mode for a percussion score (lift the `add-drum-kit-view` interim); keyboard offering untouched
- [x] 3.2 The gate's required set over strokes: freeze at each onset; release when every required stroke (selected hands/feet only) is struck **while the gate is active**; multi-piece onsets need every stroke in any order; the kick gates via the bar like any note
- [x] 3.3 No hold semantics: an early strike does not pre-satisfy; each onset needs its own strokes — widget test through the player seam (a notifier-only test cannot see the Ticker seam; see `player-ticker-seam-testing`)
- [x] 3.4 Hidden-limb exclusion: with **hands** selected, a kick-only onset has an empty required set and the gate advances (test); with **feet** selected, hand onsets likewise
- [x] 3.5 Indicator: expected pads (and the kick pedal when a kick is required) pulse while blocked, stop on release; no overlay, no banner
- [x] 3.6 Confirm the free-run countdown behaves for percussion exactly as for keyboard (fresh start shows it, Wait Mode start does not) — no code expected, a regression test

## 4. Gauge, summary, replay (Flutter)

- [x] 4.1 Score chip in the top bar during a scored percussion run in the cascade (and the percussion notation modes); hidden when no run; nothing floats over the lanes or the bar
- [x] 4.2 Hit feedback anchors: a hand stroke's spark on its lane at the hit line, a kick's on the full-width bar; combo increments/resets identically to keyboard; tier escalation driven by the percussion percentage
- [x] 4.3 Summary modal for a percussion run: two-dimension breakdown (no sustain row — absent, not rendered empty), per-verdict counts, best combo, sub-scores per mode; explicit-choice behaviour unchanged
- [x] 4.4 Mistake replay: render on the percussion scrolling staff (`add-drum-notation-render` engraving), mistake list without a sustain category, the player's own strokes sounded through the percussion channel (`add-drum-audio-channel`), missed strokes silent
- [x] 4.5 Localise any new strings (fr/en), through the ARB template with no translation drift

## 5. Difficulty (crawler + re-grade)

- [x] 5.1 `crates/score-crawler/src/difficulty.rs`: instrument switch in `estimate`/`difficulty_score` — percussion estimates from stroke density (unpitched notes/measure), tempo, fastest subdivision, limb simultaneity (simultaneous distinct pieces; two-voice writing), kit breadth; keyboard path unchanged
- [x] 5.2 Calibrate the percussion thresholds against the authored bundled drum scores (~4 short drum scores across the beginner / intermediate / advanced folders — `add-instrument-context` 6.1): each bundled score's tier must come out of the heuristic as authored, or the thresholds move, not the scores
- [x] 5.3 Tests: a dense fast multi-limb groove is not Beginner; a two-piece whole-note exercise is; a source grade still wins and is never overwritten by the heuristic
- [x] 5.4 Re-grade pass over existing percussion rows with `level_source = 'heuristic'` (follow the `backend/music/src/backfill.rs` shape — idempotent, resumable); `source`/`manual` rows untouched; run it before the backend lift (7.x) so the first paid percussion runs meet honest weights

## 6. Backend — lift the interim ingest sites (each deliberately, each tested)

- [x] 6.1 `backend/music/src/play_grpc.rs:310`: lift the **engagement** branch — a percussion play records coverage engagement (idempotent per user/piece as today)
- [x] 6.2 `play_grpc.rs:339`: lift the **award** branch — a percussion session pays through the unchanged floor/curve/weight/cap; keyed on the session id (exactly-once under at-least-once ingest)
- [x] 6.3 `play_grpc.rs:348`: lift the **streak** branch — a percussion play credits the local day
- [x] 6.4 `play_module.rs:134`: lift the **leaderboard sink** branch — percussion sessions maintain per-piece and season bests (monotone, idempotent, integrity-gated)
- [x] 6.5 `module.rs:1360` and `module.rs:1434`: lift the **daily-access** exemptions — a percussion open consumes a day slot and a percussion piece can be day-locked, upsell state included
- [x] 6.6 Update the interim tests that pinned the fail-closed behaviour (`play_grpc.rs` "a_percussion_session_is_stored_but_engages_no_artifact", the module day-lock exemption tests) to assert the lifted behaviour instead
- [x] 6.7 **Nothing retroactive**: test that percussion sessions stored before the lift stay inert — no pass re-scans them, and re-ingesting an interim session id is the idempotent no-op it always was (stored ⇒ no double record, no late award)
- [x] 6.8 Idempotence per site: a resent percussion session pays once, bests not duplicated, one streak day, one engagement row
- [x] 6.9 Integrity checks: confirm the ingest invariants (sub-score ranges, onset counts vs the piece's note count — which counts unpitched notes since `add-drums-access`) hold for percussion results, with a test for an implausible percussion result being board-excluded

## 7. Backend — the leaderboard read surface joins the enforcement

- [x] 7.1 Wire the instrument lookup + the existing `caller_may_see_percussion` predicate into the per-piece leaderboard reads (`leaderboard_grpc.rs` / `leaderboard_module.rs`): `GetLeaderboard` for a percussion piece answers an ineligible caller exactly as for a piece with no board
- [x] 7.2 `GetMyStandings` (the batched card/summary read): percussion pieces contribute no standing and no board signal for an ineligible caller — indistinguishable from boardless pieces in the same batch
- [x] 7.3 Leave the global reads (`GetGlobalLeaderboard`, `ListGlobalSeasons`) ungated — they disclose players and scores, never pieces — with a test that a percussion-fed standing is still readable by an ineligible caller and exposes no piece identity
- [x] 7.4 One ineligible-case test per gated read path (the predicate tested in isolation proves nothing about the call sites), plus the eligible-caller-unaffected case

## 8. Gates

- [x] 8.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`
- [x] 8.2 `cargo llvm-cov --workspace --fail-under-lines 80` with the repo's usual ignore regex
- [x] 8.3 `melos run analyze`, `dart format`, `dart run custom_lint` clean
- [x] 8.4 `cd apps/music && flutter test --coverage --exclude-tags golden`, coverage ≥ 80%
- [x] 8.5 `openspec validate add-drum-scoring --strict`

## 9. Manual verification (on-device, with a real e-kit where noted)

- [x] 9.1 As a beta member with an e-kit: play a bundled drum score in free run — gauge live in the cascade, sparks on lanes and bar, summary shows two dimensions and per-verdict counts, no sustain row — VALIDÉ 2026-08-25 en **production**, depuis un compte membre de la campagne `midi-drums` : run libre sur partition batterie, jauge vivante dans la cascade et résumé conforme
- [ ] 9.2 Wait Mode on a drum score: gate blocks at each onset, expected pads pulse, a strike of the right piece releases, a wrong piece does not; a kick+snare coincidence needs both; with **hands** selected a kick-only onset is skipped
- [ ] 9.3 Hi-hat shading on hardware without a controller: a closed stroke completes a run written with open hi-hats, and the summary shows the capped verdicts
- [x] 9.4 Feel pass on the reused windows: drive a rudiment-heavy score (flams, doubles) at tempo on phone and tablet before declaring the windows final — the constants are expected to move, the spec is not — VALIDÉ 2026-08-24 : passe de feel sur les fenêtres validée sur appareil ; la tolérance de frappe (150 ms, plancher temps réel) est jugée bonne
- [x] 9.5 After the run: points awarded per the acknowledgement, the piece's board shows the best, streak day credited, day slot consumed (gate flag on in staging); repeat the run and confirm the diminishing award and the monotone best — VALIDÉ 2026-08-25 en production (backend 0.21.1)
- [x] 9.6 A selective (measure-range) drum run: plays with the gate, ends with no summary, no award beyond the daily practice acknowledgement — unscored by construction — VALIDÉ 2026-08-25 en production (backend 0.21.1)
- [x] 9.7 From a **non-staff, non-member** account (staff match every beta scope — prove nothing from an admin session): the percussion piece's board read answers as boardless, the batched card standings show nothing for it, and the global board still opens — VALIDÉ 2026-08-25 en production (backend 0.21.1)
- [ ] 9.8 Percussion sessions recorded before the lift (staging holds some from the interim): confirm they engaged nothing and still engage nothing after the lift
- [ ] 9.9 Difficulty spot-check: the authored bundled scores grade at their tiers; the re-graded catalog percussion rows hold plausible levels and every `source`/`manual` grade is untouched
- [x] 9.10 Keyboard regression: a keyboard run's gauge, summary, awards, boards, streak and daily access behave exactly as before, and the keyboard sync% of a reference run is unchanged

## Addendum — the stroke tolerance window (2026-08-24) — VALIDÉ 2026-08-25 en production (backend 0.21.1)

- [x] A.1 `kStrokeToleranceMs`, one number in `drum_kit.dart`, shared by the Wait Mode gate and by every surface that lights a stroke
- [x] A.2 The gate credits a stroke played within the window before the onset, stamped on the **playhead's** clock, and spends it so one stroke never validates two onsets
- [x] A.3 Tests: a stroke a hair early walks the playhead through the onset; a stroke earlier than the window still does not; a credited stroke is spent
- [x] A.4 The window has a floor in real time — `strokeToleranceMsAt(speed)` widens it in musical time above 100 % so it never tightens in wall-clock terms; the surfaces size their lighting window with the same function, and a test pins double speed
- [ ] A.5 Feel pass: is 150 ms the right size on a real kit at tempo? (the speed question is settled — a floor, not a scaling)
