## Why

Note pitch is currently derived from a note's explicit `<alter>` element only.
The parsed key signature (`<fifths>`) is used to *draw* the armure but never to
compute pitch, and measure-local accidentals are not carried forward. As a
result, a note that lacks an explicit `<alter>` but is altered by the key
signature (or by an accidental placed earlier in the same measure) is read as
**natural** and played on the wrong soundfont pitch. Scores exported by
MuseScore/Finale/Sibelius happen to work because they emit `<alter>` on every
affected note, but minimal or non-conforming user-imported MusicXML — which
relies on the armure alone — sounds wrong notes.

## What Changes

- Apply the key signature at parse time in `musicxml-core` **only for scores that
  carry no alteration data at all** — no `<alter>` and no `<accidental>` anywhere
  in the document. For such a score, each pitched note's alteration is set from
  its measure's key signature (`<fifths>`), so a minimal/non-conforming file that
  drew an armure but never marked notes now sounds correct. Downstream consumers
  (audio playback `midiOfPitch`, ambitus scan `pitch_to_midi`, staff renderer
  `_midiOf`, and the shared web `playback::midi_of_pitch` added in #138) keep
  reading `alter` and become correct with no duplicated logic — single source of
  truth.
- The inference is document-wide gated: the **first** explicit `<alter>` or
  `<accidental>` anywhere disables it entirely, and every note's alteration is
  taken verbatim from its `<alter>` (0 when absent) — exactly today's behavior.
- Mid-piece key changes are honored: the key signature applied to a note is the
  one in force for its measure.

### Why so narrow (empirical finding during design)

A broader "resolve effective alteration per note" (apply the key signature and
running measure accidentals to any un-`<alter>`ed note) was prototyped and
**rejected**: replayed over the shipping scores it changed 8 notes' pitch. Real
conforming exporters (MuseScore) encode measure accidentals by writing an
accidental **once** and leaving later same-pitch notes bare, so a bare note is
often a legitimately natural note in an accidental context — and the voice/staff
scope of that persistence is itself ambiguous across real files (a sharp that
must NOT cross voices in one score vs. a natural that MUST cross voices in
another). No per-note policy that touches bare notes is regression-free. The only
regression-proof rule is: infer the key signature **only when the document has no
alteration data whatsoever** (conforming files always have some, so they are
never touched).

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `score-notation`: the **Note Extraction** requirement changes — the alteration
  reported on a note event is the *effective* alteration (explicit alter →
  measure accidental → key signature → natural), not the raw `<alter>` value.
  Adds accidental-persistence-within-measure and key-signature-inference
  behavior to the parser's public output.

## Impact

- **Code**: `crates/musicxml-core` parser/model (effective-alteration resolution
  during measure parsing; key-signature → per-step alteration table; per-measure
  accidental map keyed by `(step, octave)`). No API-shape change — the existing
  `Pitch.alter` field now carries the resolved value.
- **Downstream (no logic change, corrected for free)**:
  `apps/music/lib/state/notation_playback.dart` (`midiOfPitch`),
  `crates/musicxml-core/src/meta.rs` (`pitch_to_midi`, ambitus),
  `apps/music/lib/painters/partition_painter.dart` (`_midiOf`).
- **Tests**: new Rust unit cases (note without `<alter>` under `fifths=-3`
  sounds flat; explicit natural stays natural; accidental persists within a
  measure on the same `(step, octave)` then resets next measure; a conforming
  fixture stays byte-for-byte identical in pitch). Keep line coverage ≥ 80%
  (Rust + Flutter). Existing `prelude_e_minor` / arabesque behavior unchanged.
- **No breaking API**; no migration.
