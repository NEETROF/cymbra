## Context

`musicxml-core` is the shared streaming parser (`crates/musicxml-core`). Its
`Handler` accumulates notes per measure; key signature is a running field
`self.key_fifths` updated on `<fifths>`, starting at 0 and persisting across
measures. Each `Pitch { step, octave, alter }` sets `alter` only when an
`<alter>` element is present (0 otherwise); a note also carries
`accidental: Option<String>`.

Four consumers derive a MIDI number with the identical formula
`(octave+1)*12 + semitone(step) + alter` and all trust `alter`:
`playback::midi_of_pitch` (Rust, shared with the web WASM console since #138),
`meta::pitch_to_midi` (ambitus), Dart `notation_playback.midiOfPitch` (app
playback), and Dart `partition_painter._midiOf` (rendering). None applies the key
signature.

**Empirical finding that shaped this design.** A first prototype resolved each
un-`<alter>`ed note from the key signature plus running measure accidentals.
Replayed over the five shipping scores it changed **8 notes' pitch** — a
regression. The cause: conforming exporters (MuseScore) encode a measure's
accidentals by writing an accidental **once** and leaving later same-pitch notes
bare, so a bare note frequently means "natural in this measure's accidental
context," not "apply the key signature." Worse, the persistence scope is
inconsistent across real files — in `prelude_e_minor` m13 a G♯ in voice 1 must
**not** carry to a bare G in voice 2, while in `arabesque` m63 a C♮ in voice 2/5
**must** carry to a bare C in voice 1. No per-note rule that touches bare notes is
regression-free. Therefore inference is restricted to scores with **no alteration
data at all**, which conforming files never are.

## Goals / Non-Goals

**Goals:**
- Make a purely minimal score (a key signature drawn, but no `<alter>` and no
  `<accidental>` anywhere) sound its armure.
- Zero regression on any score that carries alteration data (all real exporter
  output), proven by replaying the resolution over the shipping scores.
- One source of truth in `musicxml-core` so all four `alter` consumers — including
  the shared web `playback::midi_of_pitch` — are correct with no edits.

**Non-Goals:**
- No general measure-accidental engine, no cross-voice/staff persistence, no
  attempt to re-derive bare notes in files that DO carry alteration data. Those
  are exactly the ambiguous cases the finding above rules out.
- No change to the `Pitch`/`NoteEvent` public shape or the FFI/Dart API — only the
  `alter` field's value changes, and only for otherwise-empty-of-alteration files.
- No microtones beyond `<alter>`; inference only ever contributes ±1.

## Decisions

### D1 — Gate inference on a document-wide "has any alteration" flag

During parse, set a single `saw_alteration: bool` the first time any `<alter>` or
any `<accidental>` is seen. Inference runs only when this stays false for the
whole document. Rationale: this is the one property that cleanly separates
"minimal file relying on the armure" from every conforming file (which always
emits at least one alteration), and it needs no per-note ambiguity.
*Alternative rejected:* per-note key-signature/measure-accidental resolution — the
prototype that regressed 8 shipping notes.

### D2 — Apply after parse, per-measure key signature

Record the key signature in force for each emitted measure in a parser-side
`measure_fifths: Vec<i32>` (parallel to `measures`, pushed in `finish_measure`).
In `into_document`, if `!saw_alteration`, walk each measure with its fifths and
set every pitched note's `alter = key_signature_alter(fifths, step)`. This is a
clean single post-pass, keeps the public model unchanged, and honors mid-piece
key changes without a full accidental state machine.
*Alternative rejected:* apply the final `key_fifths` uniformly — wrong for a
minimal score with a mid-piece key change.

### D3 — Key-signature → per-step alteration table

A pure `key_signature_alter(fifths: i32, step: char) -> i32` in a small private
`pitch_alter` module: sharps order `F C G D A E B` (+1), flats order
`B E A D G C F` (-1), count clamped to 7, unaffected steps 0. Host-testable, no
state. It is the only surviving piece of the original design; the per-note
`resolve`/measure-accidental machinery is dropped.

### D4 — Downstream untouched

`midi_of_pitch`, `pitch_to_midi`, `midiOfPitch`, `_midiOf` are not edited. They
read the corrected `alter` for free. Because the public Rust API shape is
unchanged, no `flutter_rust_bridge_codegen` regeneration is required.

## Risks / Trade-offs

- **[Regression on a real score]** → Structurally impossible: any `<alter>`/
  `<accidental>` disables inference, and every conforming file has some. Backed by
  a replay over all shipping scores asserting 0 pitch changes.
- **[A partially-marked minimal file]** (marks some chromatic accidentals but
  relies on the armure for the rest) → still reads bare notes as natural (no
  inference), i.e. unchanged from today. This is inherently ambiguous input; we
  do not guess. Documented limitation.
- **[Minimal file with mid-piece key change]** → handled by the per-measure
  `measure_fifths` (D2).
- **[Coverage]** → `key_signature_alter` and the gated application are directly
  unit-tested; Dart is unaffected (its existing `alter`-consuming tests stand).

## Migration Plan

Pure parser enhancement; no data migration, no API break, ships in one change.
Rollback = revert the commit; downstream formulas are untouched. Public Rust API
unchanged → no bridge regeneration.

## Open Questions

- None outstanding. (The earlier question about a defensive `<accidental>`-without-
  `<alter>` mapping is moot: any `<accidental>` now disables inference entirely.)
