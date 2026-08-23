## 1. Fixtures

- [x] 1.1 Add a percussion MusicXML fixture to `crates/musicxml-core` tests: a `<part-list>` declaring `<score-instrument>` + `<midi-instrument><midi-unpitched>` for kick (element 37 → GM 36), snare (39 → GM 38) and closed hi-hat (43 → GM 42) — the **one-based** values real exporters write, so the fixture catches the off-by-one a self-consistent fixture would hide; a percussion clef; two measures of a rock groove written in two voices (hi-hat + snare stems up, kick stems down)
- [x] 1.2 Add fixtures covering the degraded cases: an unpitched note whose `<instrument id>` is absent from the part list; a part list carrying no `<score-instrument>` at all; a note with no `<instrument>` element in a part declaring exactly one instrument (must resolve via the sole-instrument fallback) and in a part declaring several (must stay unknown); an empty `<unpitched/>` (must produce a note event at the middle-line default placement); and a degenerate note (none of pitch/unpitched/rest) that must be skipped with its duration still advancing the position
- [x] 1.3 Mirror fixture 1.1 as a Dart test fixture under `apps/music/test/` so the schedule-parity test (5.3) has the same input on both sides
- [x] 1.4 Add a tied-cymbal fixture: a percussion part tying a crash or ride note across a barline, exercising the unpitched tie-merge rule (one prolonged note keyed by voice + resolved GM number); mirror it under `apps/music/test/` so the parity test (5.3) covers ties on both sides

## 2. Model — `crates/musicxml-core/src/model.rs`

- [x] 2.1 Add the unpitched channel to `NoteEvent`: written position (display step + octave) and the resolved General MIDI number, kept distinct from `pitch`; document that the written position is a staff placement and MUST NOT be read as a sounding pitch
- [x] 2.2 Retain a note's `<instrument id>` on `NoteEvent`
- [x] 2.3 Change `Clef.sign` from `char` to an enum with `G`, `F`, `C`, `Percussion`; any other MusicXML sign (`TAB`, `jianpu`, `none`) or an unrecognised one leaves the staff at its default clef (**BREAKING** — crate API)
- [x] 2.4 Add the part-list instrument table to `ScoreDocument` (instrument id → General MIDI percussion number, plus the instrument name when declared)
- [x] 2.5 Keep every new field `u32`/`i32`/`f64` and serde-derivable under the `serde` feature, per the module's existing constraints (no `u64`, so the bridge avoids Dart `BigInt`; the wasm consumer needs Serialize/Deserialize)

## 3. Parser — `crates/musicxml-core/src/lib.rs`

- [x] 3.1 Parse `<part-list>` → `<score-part>` → `<score-instrument>` / `<midi-instrument>` / `<midi-unpitched>` into the instrument table
- [x] 3.2 Parse `<unpitched>` with `display-step` / `display-octave` into the unpitched channel; a note is exactly one of pitched, unpitched, or rest
- [x] 3.3 Parse `<instrument>` on a note and resolve it against the instrument table at parse time — the General MIDI number is the **one-based** `midi-unpitched` element value minus one; a note carrying no `<instrument>` element falls back to the part's sole declared instrument when exactly one carries a `midi-unpitched`, and stays unknown when several are declared; leave the General MIDI number unknown when unresolvable — never infer it from the written position
- [x] 3.4 Accept `<clef><sign>percussion</sign>`; leave an unrecognised sign at the staff default without failing the parse
- [x] 3.5 Unit tests against fixtures 1.1 and 1.2 covering: written position captured, instrument id resolved to its GM number (element value minus one — assert fixture element 39 reads as GM 38), unresolvable id left unknown, no-reference note resolved via the sole-instrument fallback and left unknown among several, empty `<unpitched/>` placed at the middle-line default, degenerate note skipped with its duration still advancing the position, absent part list still parses, percussion clef reported, unknown sign degrades, pitched parsing unchanged

## 4. Facets — `crates/musicxml-core/src/meta.rs`

- [x] 4.1 Derive the instrument classification from the first part's parse alone, never from a filename or part name: percussion when every non-rest note is unpitched, keyboard when every non-rest note is pitched, unknown otherwise (mixed content, or no notes at all)
- [x] 4.2 Leave the ambitus (lowest/highest pitch) unknown for a percussion score rather than deriving it from written staff positions
- [x] 4.3 Leave `note_count`, `is_piano` and `validate()` **untouched** — a percussion score must still be refused by the gate in this change; assert that with a test so the boundary is explicit and a later change cannot open it by accident
- [x] 4.4 Tests for 4.1–4.3

## 5. Playback schedule and its Dart mirror

- [x] 5.1 `playback.rs::schedule()`: emit unpitched notes **only when the score classifies as percussion**, carrying their resolved General MIDI number, timed by the existing rules; merge tied unpitched chains into one prolonged note keyed by (voice, resolved GM number) and omit a chain whose GM number is unresolved; omit an unresolvable note rather than fabricating a number, and confirm surrounding notes keep their computed times; assert a mixed pitched+unpitched score schedules exactly as today (unpitched skipped)
- [x] 5.2 `apps/music/lib/state/notation_playback.dart`: apply the identical rule in `notationToTimedNotes` (the two implementations are deliberate mirrors), including the percussion-classification condition and the unpitched tie key (voice + GM number — the pitched tie key derives from `midiOfPitch`, uncomputable here)
- [x] 5.3 Parity test: schedule fixtures 1.1 and 1.4 on both sides and assert the same notes, start times, durations and General MIDI numbers — a drift here makes the back-office preview time a score differently from the app
- [x] 5.4 Confirm tie-merging and chord-onset behaviour are untouched for pitched scores (regression guard on the existing tie-chain logic)

## 6. Consumers — compile-time ripple

- [x] 6.1 `apps/music/rust`: wrap the new model fields at the FFI seam, then run `flutter_rust_bridge_codegen generate`
- [x] 6.2 `apps/music/lib`: keep `TimedNote.clefSign` a `String` derived from the new enum through one letter-mapping helper, and route the partition painter's four bridged-`Clef.sign` read sites (and its `Clef` literals) through it
- [x] 6.3 `crates/musicxml-wasm` and `crates/audio-wasm`: fix against the changed `Clef.sign` / `NoteEvent` API; behaviour unchanged
- [x] 6.4 `crates/score-crawler`: fix against the changed API; confirm the ingest gate still refuses a percussion score (unchanged behaviour)
- [x] 6.5 `backend/music`: fix against the changed API; confirm no DB or proto change is required
- [x] 6.6 `apps/back-office`: run `yarn gen:wasm` and hard-refresh — a stale wasm silently renders against the old parser and reads as a code bug, not a build problem

## 7. Gates

- [x] 7.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`
- [x] 7.2 `cargo llvm-cov --workspace --fail-under-lines 80` with the repo's usual ignore regex
- [x] 7.3 `melos run analyze`, `dart format`, and `dart run custom_lint` clean
- [x] 7.4 `cd apps/music && flutter test --coverage --exclude-tags golden`, coverage ≥ 80%
- [x] 7.5 `openspec validate add-unpitched-notation --strict`

## 8. Manual verification

- [ ] 8.1 Confirm the app is behaviourally unchanged: a piano score opens, plays and renders exactly as before in all three modes
- [ ] 8.2 Confirm a drum MusicXML file is still refused by the upload preview (the gate has not moved in this change)
- [ ] 8.3 Confirm the back-office notation preview still renders a piano score correctly after the wasm rebuild (guards against 6.6 being skipped)
