## 1. Fixtures

- [ ] 1.1 Add a percussion MusicXML fixture to `crates/musicxml-core` tests: a `<part-list>` declaring `<score-instrument>` + `<midi-instrument><midi-unpitched>` for kick (36), snare (38) and closed hi-hat (42); a percussion clef; two measures of a rock groove written in two voices (hi-hat + snare stems up, kick stems down)
- [ ] 1.2 Add a second fixture covering the degraded cases: an unpitched note whose `<instrument id>` is absent from the part list, and a part list carrying no `<score-instrument>` at all
- [ ] 1.3 Mirror fixture 1.1 as a Dart test fixture under `apps/music/test/` so the schedule-parity test (5.3) has the same input on both sides

## 2. Model — `crates/musicxml-core/src/model.rs`

- [ ] 2.1 Add the unpitched channel to `NoteEvent`: written position (display step + octave) and the resolved General MIDI number, kept distinct from `pitch`; document that the written position is a staff placement and MUST NOT be read as a sounding pitch
- [ ] 2.2 Retain a note's `<instrument id>` on `NoteEvent`
- [ ] 2.3 Change `Clef.sign` from `char` to an enum with `G`, `F`, `C`, `Percussion`; any other MusicXML sign (`TAB`, `jianpu`, `none`) or an unrecognised one leaves the staff at its default clef (**BREAKING** — crate API)
- [ ] 2.4 Add the part-list instrument table to `ScoreDocument` (instrument id → General MIDI percussion number, plus the instrument name when declared)
- [ ] 2.5 Keep every new field `u32`/`i32`/`f64` and serde-derivable under the `serde` feature, per the module's existing constraints (no `u64`, so the bridge avoids Dart `BigInt`; the wasm consumer needs Serialize/Deserialize)

## 3. Parser — `crates/musicxml-core/src/lib.rs`

- [ ] 3.1 Parse `<part-list>` → `<score-part>` → `<score-instrument>` / `<midi-instrument>` / `<midi-unpitched>` into the instrument table
- [ ] 3.2 Parse `<unpitched>` with `display-step` / `display-octave` into the unpitched channel; a note is exactly one of pitched, unpitched, or rest
- [ ] 3.3 Parse `<instrument>` on a note and resolve it against the instrument table at parse time; leave the General MIDI number unknown when unresolvable — never infer it from the written position
- [ ] 3.4 Accept `<clef><sign>percussion</sign>`; leave an unrecognised sign at the staff default without failing the parse
- [ ] 3.5 Unit tests against fixtures 1.1 and 1.2 covering: written position captured, instrument id resolved to its GM number, unresolvable id left unknown, absent part list still parses, percussion clef reported, unknown sign degrades, pitched parsing unchanged

## 4. Facets — `crates/musicxml-core/src/meta.rs`

- [ ] 4.1 Derive the instrument classification (keyboard / percussion / unknown) from the parse alone, never from a filename or part name
- [ ] 4.2 Leave the ambitus (lowest/highest pitch) unknown for a percussion score rather than deriving it from written staff positions
- [ ] 4.3 Leave `note_count`, `is_piano` and `validate()` **untouched** — a percussion score must still be refused by the gate in this change; assert that with a test so the boundary is explicit and a later change cannot open it by accident
- [ ] 4.4 Tests for 4.1–4.3

## 5. Playback schedule and its Dart mirror

- [ ] 5.1 `playback.rs::schedule()`: emit unpitched notes carrying their resolved General MIDI number, timed by the existing rules; omit an unresolvable note rather than fabricating a number, and confirm surrounding notes keep their computed times
- [ ] 5.2 `apps/music/lib/state/notation_playback.dart`: apply the identical rule in `notationToTimedNotes` (the two implementations are deliberate mirrors)
- [ ] 5.3 Parity test: schedule fixture 1.1 on both sides and assert the same notes, start times, durations and General MIDI numbers — a drift here makes the back-office preview time a score differently from the app
- [ ] 5.4 Confirm tie-merging and chord-onset behaviour are untouched for pitched scores (regression guard on the existing tie-chain logic)

## 6. Consumers — compile-time ripple

- [ ] 6.1 `apps/music/rust`: wrap the new model fields at the FFI seam, then run `flutter_rust_bridge_codegen generate`
- [ ] 6.2 `apps/music/lib`: keep `TimedNote.clefSign` a `String` derived from the new enum, so the staff and partition painters are untouched
- [ ] 6.3 `crates/musicxml-wasm` and `crates/audio-wasm`: fix against the changed `Clef.sign` / `NoteEvent` API; behaviour unchanged
- [ ] 6.4 `crates/score-crawler`: fix against the changed API; confirm the ingest gate still refuses a percussion score (unchanged behaviour)
- [ ] 6.5 `backend/music`: fix against the changed API; confirm no DB or proto change is required
- [ ] 6.6 `apps/back-office`: run `yarn gen:wasm` and hard-refresh — a stale wasm silently renders against the old parser and reads as a code bug, not a build problem

## 7. Gates

- [ ] 7.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 7.2 `cargo llvm-cov --workspace --fail-under-lines 80` with the repo's usual ignore regex
- [ ] 7.3 `melos run analyze`, `dart format`, and `dart run custom_lint` clean
- [ ] 7.4 `cd apps/music && flutter test --coverage --exclude-tags golden`, coverage ≥ 80%
- [ ] 7.5 `openspec validate add-unpitched-notation --strict`

## 8. Manual verification

- [ ] 8.1 Confirm the app is behaviourally unchanged: a piano score opens, plays and renders exactly as before in all three modes
- [ ] 8.2 Confirm a drum MusicXML file is still refused by the upload preview (the gate has not moved in this change)
- [ ] 8.3 Confirm the back-office notation preview still renders a piano score correctly after the wasm rebuild (guards against 6.6 being skipped)
