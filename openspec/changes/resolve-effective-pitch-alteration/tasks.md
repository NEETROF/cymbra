## 1. Key-signature alteration table (pure)

- [x] 1.1 Add a private `pitch_alter` module in `musicxml-core` with `key_signature_alter(fifths: i32, step: char) -> i32` (sharps `F C G D A E B` → +1, flats `B E A D G C F` → -1, count clamped to 7, else 0)
- [x] 1.2 Unit tests: `fifths=0` → all 0; `fifths=-3` → B/E/A = -1 and C/D/F/G = 0; `fifths=2` → F/C = +1 and others 0; `fifths=±10` clamps to ±7 without panic; case-insensitive step

## 2. Document-wide "has any alteration" gate

- [x] 2.1 Add `saw_alteration: bool` to the parse handler; set it true on the first `<alter>` and on the first `<accidental>`
- [x] 2.2 Record the key signature in force per emitted measure in `measure_fifths: Vec<i32>` (pushed in `finish_measure`, parallel to `measures`)

## 3. Gated key-signature application (post-parse)

- [x] 3.1 In `into_document`, when `!saw_alteration`, walk each measure with its `measure_fifths` and set every pitched note's `alter = key_signature_alter(fifths, step)` (skip when fifths == 0)
- [x] 3.2 Leave notes untouched when `saw_alteration` is true (verbatim `<alter>`, 0 when absent) — today's behavior

## 4. Rust tests (parser + meta)

- [x] 4.1 Unmarked score (no `<alter>`/`<accidental>`) under `fifths=-3`: B → -1, E → -1, A → -1, C/D/F/G → 0
- [x] 4.2 A single explicit `<alter>` (or `<accidental>`) anywhere disables inference: the marked note keeps its value, other bare notes stay 0
- [x] 4.3 Mid-piece key change in an unmarked score: B in a `fifths=-3` measure → -1, B in a later `fifths=0` measure → 0
- [x] 4.4 `meta.rs` ambitus: an unmarked `fifths=-3` score reports the flattened MIDI numbers (E♭4=63, A♭4=68), proving `pitch_to_midi` consumes the inferred alteration

## 5. Regression proof on shipping scores

- [x] 5.1 Confirm every shipping score (`prelude_e_minor`, `arabesque`, `twinkle`, `ode_to_joy`, `minuet_in_g`) carries alteration data, so inference never runs and no note's pitch changes (0 diffs vs. pre-change)
- [x] 5.2 Confirm `notation_playback.midiOfPitch`, `meta.pitch_to_midi`, `partition_painter._midiOf`, and shared `playback::midi_of_pitch` are unchanged and correct because they read `Pitch.alter`

## 6. Gates & specs

- [x] 6.1 Confirm the Rust public API (Pitch/NoteEvent shape) is unchanged; no `flutter_rust_bridge_codegen` regeneration needed
- [x] 6.2 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` clean; `melos run analyze` + `dart format` clean
- [x] 6.3 Coverage ≥ 80% both ecosystems; Rust + Flutter test suites green
- [x] 6.4 `openspec validate resolve-effective-pitch-alteration --strict` passes
