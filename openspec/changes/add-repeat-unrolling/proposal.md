# Add Repeat Unrolling

## Why

The notation engine ignores every MusicXML repeat construct: repeat barlines
(`<barline><repeat>`), voltas (`<ending>`), measure-repeat signs
(`<measure-style><measure-repeat>`) and D.C./D.S./segno/coda jumps are not
parsed at all. Pieces with repeats play linearly (each section once, both
voltas back-to-back), a `%` measure plays as **silence** (its content is
implicit in the file), and the engraving shows plain barlines where the score
has repeats — so the player cannot even see that something is missing. A large
share of real-world piano scores (and of the crawled catalog) uses repeats;
today the app, the back-office preview and the server-rendered audio previews
all play them wrong.

## What Changes

- **Parse the repeat structure** in the shared `cymbra-musicxml-core` crate:
  repeat barlines (forward/backward, `times`), volta brackets (ending
  numbers, start/stop/discontinue), measure-repeat (`%`) references, and
  D.C./D.S./Fine/segno/coda jumps (from `<sound>` attributes, with the
  engraved words/glyphs kept for display).
- **Compute a playback order** ("unroll") in the core crate: the sequence of
  written-measure passes a performer would play, with safety caps and a
  fall-back to written order on malformed structure. Both derivations consume
  it: the Rust `playback::schedule()` (back-office Play preview via wasm, and
  the server audio-preview render job) and the Flutter
  `notationToTimedNotes` (waterfall, scrolling staff, Wait-Mode gate,
  scoring, transport).
- **Engrave the repeat notation**: repeat barlines, volta brackets, `%`
  signs, and jump words/glyphs in the app's Partition view and scrolling
  staff, and in the back-office wasm renderer — instead of plain barlines and
  empty measures.
- **Play `%` measures** by replaying the referenced measure's content (playback
  only; the engraving keeps the sign).
- **Map played time back to written measures** so the Partition cursor, the
  measure-rewind, and practice-range selection stay correct when a written
  measure is played several times.
- **Practice runs stay linear**: a selected measure range plays the written
  measures once, without unrolling (documented, keeps the loop semantics
  unambiguous).
- **Extend the long-press notation help** to the new symbols: repeat
  barlines, volta brackets, measure-repeat sign, segno/coda/D.C./D.S./Fine —
  recorded in the hit index by both painters, explained with localized copy in
  all four app locales (en/fr/es/it, no translation drift) and added to the
  glossary.

## Capabilities

### New Capabilities

(none — the repeat semantics land in the capabilities that own each surface)

### Modified Capabilities

- `score-notation`: the parser SHALL extract the repeat structure; derived
  playback (app) SHALL follow the unrolled playback order; per-measure timing
  gains the played↔written measure mapping; the Partition/staff engraving
  SHALL draw repeat barlines, volta brackets and `%` signs.
- `web-notation-render`: the back-office wasm renderer SHALL engrave the
  repeat notation, and its playback schedule (Play preview) SHALL be the
  unrolled timeline.
- `music-score-audio-preview`: server-rendered preview clips SHALL reflect
  the played (unrolled) timeline, not the written measure sequence.
- `notation-help` (in-flight change `add-notation-help`): the long-press help
  SHALL resolve and explain the new repeat symbols, in every locale.
- `measure-range-practice`: range selection is defined over **written**
  measures; a selective run SHALL play them linearly once (no unrolling).

## Impact

- **Products**: Cymbra **Music** (app: playback, engraving, help, practice,
  scoring inputs), **back-office** (wasm notation render + Play preview —
  consumes the shared crate, no BO-specific logic beyond drawing the new
  glyphs), **backend** (audio-preview render job — consumes
  `playback::schedule()` unchanged once the crate unrolls). Cymbra ID, Live
  and the site are not impacted.
- **Code**: `crates/musicxml-core` (model + parser + playback + layout),
  `crates/musicxml-wasm` (re-export only), `apps/music/rust/src/api/musicxml.rs`
  (frb mirrors — **public API change → regenerate the bridge**),
  `apps/music/lib/state/notation_playback.dart` + painters
  (staff/partition) + `staff_hit_index`/help glossary + ARB locales,
  `apps/back-office/src/lib/notation` painter. Back-office wasm must be
  rebuilt (`yarn gen:wasm`).
- **Behavioral**: pieces with repeats get **longer** played timelines —
  `songEndMs`, note counts, scoring/leaderboard inputs and daily-allowance
  heuristics see the unrolled length; existing linearized plays of such
  pieces will not be comparable with post-change scores (accepted; boards are
  seasonal). Old cached previews of repeat-carrying pieces become stale until
  regenerated (back-office backfill filter already exists).
- **Risk**: malformed or adversarial repeat structures — the unroll is capped
  and falls back to written order rather than looping or rejecting.
