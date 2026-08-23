## Why

Cymbra Music only understands **pitched** notation. A drum score is written with
`<unpitched>` note elements, so the shared parser drops every one of its notes and
the whole file reads as empty. Every drum feature — audio, rendering, kit view, pad
mapping, catalog, scoring — is blocked behind that.

This change teaches `crates/musicxml-core` to *read* percussion notation. It
deliberately does **not** admit a drum score into the system: the validation gate
stays closed, because opening it discloses drum scores to every caller and that
requires access controls, delivered by the companion change `add-drums-access`.

Splitting it this way keeps each change safe on its own. This one is inert
end-to-end: a percussion score parses correctly when the parser is called directly,
and is refused by upload and by the crawler exactly as it is today.

## What Changes

**Shared parser (`crates/musicxml-core`)**

- Parse `<unpitched>` (`display-step` / `display-octave`) as a channel **distinct
  from** `<pitch>`. A percussion note's written position is a staff placement, not
  a sounding pitch — reusing `Pitch` would let `midi_of_pitch` compute a
  meaningless frequency from it.
- Parse the `<part-list>` instrument declarations —
  `<score-part>/<score-instrument>` and `<midi-instrument>/<midi-unpitched>` —
  which carry the mapping from a note's `<instrument id>` to its General MIDI
  percussion number (35–81). This mapping is the only authoritative link between
  a written drum note and the sound it denotes.
- Accept the percussion clef. **BREAKING (crate API):** `Clef.sign` is a `char`
  and cannot hold `percussion`; it becomes a typed clef sign.
- Derive each score's **instrument classification** (keyboard / percussion /
  unknown) from the parse alone.
- Emit unpitched notes in the playback schedule, and mirror the same rule in the
  app's `notationToTimedNotes`.

**Deliberately unchanged**

- `validate()` still rejects a score with no pitched notes, so no percussion score
  can be uploaded or crawled yet.
- `note_count` keeps its current meaning (pitched, non-rest events).
- `is_piano` keeps its current meaning and its column.
- No feature flag, no user interface, no database, no wire protocol.

**Out of scope**, each with its own change: `add-drums-access` (instrument column
and migration, the flag, backend enforcement, opening the gate, the app and
back-office filters); then percussion audio, notation rendering, kit view, pad
mapping and instrument-aware scoring.

## Capabilities

### New Capabilities

- `music-percussion-notation`: how the shared parser represents percussion
  notation — unpitched note events and their written staff position, the part-list
  instrument table mapping a note to its General MIDI percussion number, the
  percussion clef, the derived instrument classification, the presence of unpitched
  notes in the playback schedule, and the explicit fact that the validation gate
  stays closed until access controls exist.

### Modified Capabilities

- `score-notation`: the parser must also read the part list's instrument
  declarations, admit a percussion clef sign, extract an unpitched note's written
  position, and derive the playback timing of an unpitched note from its General
  MIDI number rather than from a pitch.
- `score-facets`: a score gains a derived **instrument** facet; the pitch ambitus is
  left unknown for a percussion score, whose notes carry a staff placement rather
  than a sounding pitch.

## Impact

**Products**

| Product | Consumes | New |
|---|---|---|
| **Music** (`apps/music`) | the shared parser | the Dart schedule mirror; nothing user-visible |
| **Back-office** (`apps/back-office`) | `musicxml-core` through its wasm wrapper | none — but the wasm **must** be rebuilt (`yarn gen:wasm`), or the notation preview silently renders against a stale parser and reads as a code bug |
| **ID / Live / Site / platform** | — | untouched |

**Code**

- `crates/musicxml-core`: `model.rs` (note event, clef), `lib.rs` (parser),
  `meta.rs` (instrument facet), `playback.rs` (schedule). `validate.rs` untouched.
- `crates/musicxml-wasm`, `crates/audio-wasm`, `crates/score-crawler`,
  `backend/music`: rebuild against the changed crate API; behaviour unchanged.
- `apps/music/rust`: FFI wrappers over the new model fields ⇒
  `flutter_rust_bridge_codegen generate`.
- `apps/music/lib/state/notation_playback.dart`: mirror of the schedule rule.

**Risk already retired.** `rustysynth` 1.3.6 supports percussion natively
(`PERCUSSION_CHANNEL = 9`, bank 128, GM fallback to the standard kit), so the audio
path is not a risk for the follow-up change.
