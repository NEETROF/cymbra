# Design — add-repeat-unrolling

## Context

The shared `cymbra-musicxml-core` crate parses MusicXML into `ScoreDocument`
(measures in written order) and derives playback twice:

- **Rust** `playback::schedule()` — consumed by `crates/musicxml-wasm`
  (back-office Play preview, in a Web Worker) and by the backend
  audio-preview render job.
- **Dart** `notationToTimedNotes()`
  (`apps/music/lib/state/notation_playback.dart`) — consumed by the
  waterfall, the scrolling staff, the Wait-Mode gate, scoring, and the
  transport (`measureStartMs`, `songEndMs`).

Neither the parser nor either derivation reads `<barline>/<repeat>`,
`<ending>`, `<measure-style>/<measure-repeat>` or `<sound
dacapo/dalsegno/fine/tocoda>`. Verified empirically: a repeated section plays
once, both voltas play back-to-back, and a `%` measure parses as an **empty**
measure (silent hole). The engraving (app Partition + staff, back-office wasm
renderer) draws plain barlines because the model carries no repeat data.

Constraints: frb mirrors in `apps/music/rust/src/api/musicxml.rs` must be
kept in sync with the crate model (public-API change ⇒ regenerate the
bridge); the back-office loads the crate through a locally built wasm
(`yarn gen:wasm`); Rust and Flutter both sit under the 80 % coverage gate.

## Goals / Non-Goals

**Goals:**

- One authoritative **unroll** computed in the core crate, consumed by both
  derivations — never two independent unroll implementations.
- Correct playback for the common repeat vocabulary: `‖: :‖` (with `times`),
  voltas 1./2. (incl. multi-measure and discontinue), `%` measure repeats,
  D.C./D.S. al Fine / al Coda driven by `<sound>` attributes.
- Engraving of the repeat notation on every renderer (app Partition, app
  scrolling staff, back-office wasm), and long-press help for each new glyph.
- Robustness: malformed structure degrades to written order, never loops.

**Non-Goals:**

- No editing/authoring of repeats; read-only fidelity.
- No per-pass dynamics/tempo differences (`<sound>` per-pass overrides
  beyond the jump attributes).
- No unrolling inside **practice** selective runs (explicitly linear).
- No re-scoring or migration of historical plays/leaderboard entries.

## Decisions

### D1 — Model: repeat data lives on the measure, written order is preserved

`NotationMeasure` gains: `barline_left` / `barline_right`
(`RepeatMark { forward | backward { times } }` + bar style), `ending`
(`Volta { numbers, kind: start|stop|discontinue }`), `measure_repeat_of:
Option<u32>` (index of the written measure whose content this `%` measure
replays; slash count for the glyph), and `jump` markers (segno, coda,
to-coda, D.C./D.S./Fine words + their `<sound>` semantics). `ScoreDocument`
keeps `measures` in **written** order — every existing consumer (layout,
Partition, validation, crawler) is untouched by default.

*Alternative rejected*: materializing the unroll into a longer `measures`
list. It would silently break engraving (repeats drawn twice, `%` lost),
measure numbering, and the practice-range/measure tables.

### D2 — One unroll in the core crate: `play_order()`

New pure function `play_order(&ScoreDocument) -> Vec<PlayedMeasure>` with
`PlayedMeasure { written_index: u32, pass: u32 }`. Resolution: standard
performance rules — forward/backward repeats (honouring `times`), volta
selection by pass number, one D.C./D.S. jump honoured with repeats **not**
re-taken after the jump (`<sound>` `forward-repeat`/`fine`/`tocoda`
respected when present). Safety caps: total played measures ≤ 8× written (and
≤ 4096); any inconsistency (unmatched backward repeat, volta without repeat,
jump target missing, cap hit) ⇒ return written order 1:1 (the exact current
behavior). The result is exposed through frb and the wasm `schedule` payload.

*Alternative rejected*: implementing the unroll in Dart and in TS/wasm
separately — two rule engines drifting apart; the whole point is one truth.

### D3 — Both derivations iterate `play_order`, tie merge follows played adjacency

`playback::schedule()` (Rust) and `notationToTimedNotes()` (Dart) walk
`play_order` instead of `measures`, accumulating time per **played** measure.
Tie chains merge on played adjacency in absolute played divisions — a tie
written across a barline merges only when the two measures are also adjacent
in the played order; otherwise the continuation falls back to a playable note
(the existing dangling-stop rule, unchanged). `%` measures replay the
referenced measure's notes at the played slot (pure playback copy — the
render-only tie/rest channels and engraving still see the written `%`).

### D4 — Played↔written mapping is first-class player state

`measureStartMs` becomes the **played**-measure table (it is what the
playhead consumes), joined by `writtenMeasureOf: List<int>` (same length).
The Partition cursor highlights the written measure of the current played
slot (cursor jumps back on a repeat — expected); measure-rewind steps through
played slots; the armure table stays aligned with played slots. Practice
range selection keeps operating on **written** measures (taps on the
Partition already resolve written measures) and a selective run plays them
linearly once — the gate/loop semantics stay unambiguous and scorer-free as
today.

### D5 — Engraving: written order + repeat glyphs; the scrolling staff draws per pass

Page-style renderers (app Partition, back-office wasm) draw written order
once, adding repeat barlines (thick/thin + dots), volta brackets with
numbers, `%` signs, and segno/coda/words glyphs. The time-scrolled staff
derives its bar lines from played slots, so a repeated measure scrolls past
once **per pass**, with its repeat barlines drawn each time and only the
volta actually played on that pass shown — the unrolled timeline makes this
fall out naturally.

### D6 — Help: new `SymbolDescriptor` kinds recorded by both painters

New kinds: repeat barline (forward/backward), volta bracket, measure-repeat
sign, segno, coda, jump words (D.C./D.S./Fine). Both the staff painter and
the Partition painter record their rects; the glossary gains one entry per
kind; copy lands in **all four** ARB locales in the same change
(no-translation-drift rule) and in the help/tips glossary.

### D7 — Back-office and server preview ride the crate

The BO Play preview and the backend audio-preview job already call
`playback::schedule()`; they unroll automatically once the crate lands. BO
work is limited to drawing the new glyphs in `lib/notation/painter.ts` and
rebuilding the wasm. Stale previews of repeat-carrying pieces are
regenerated through the existing back-office backfill filter.

## Risks / Trade-offs

- [Malformed/adversarial repeat structure] → capped unroll + 1:1 fallback;
  property tests fuzz mismatched repeats; the fallback is the exact current
  behavior, so nothing regresses.
- [Longer pieces change scoring/rewards/allowance inputs] → accepted:
  seasonal boards reset naturally; degressive play-rewards already scale by
  note count; called out in the proposal.
- [Ties written across a repeat jump] → never merged (played-adjacency
  rule); worst case a re-attack, which is what a performer does after a jump.
- [frb/wasm API change] → regenerate bridge + `yarn gen:wasm`; both are
  existing documented steps; CI builds both.
- [Cursor jumping backward on the Partition] → per-system auto-scroll
  already tolerates line jumps; add a widget test for a backward jump.
- [`%` measure referencing a `%` measure (chains)] → resolve transitively at
  parse time with a depth cap; beyond the cap ⇒ empty measure (today's
  behavior).

## Open Questions

- Multi-`times` voltas ("1.–3." brackets) — supported by parsing the number
  list; passes beyond the listed voltas replay the last bracket. Confirm on
  corpus examples during implementation.
- Whether the crawler's validation should flag scores whose unroll hits the
  fallback (nice-to-have surfacing, not required for this change).
