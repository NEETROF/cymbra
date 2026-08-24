## Why

Seven changes in, a drum score is visible to its audience (`add-drums-access`),
reads correctly (`add-drum-kit-view`, `add-drum-notation-render`), sounds like a
kit (`add-drum-audio-channel`) and answers to the sticks
(`add-drum-input-mapping`) — and still **counts for nothing**. The scorer is
keyboard-shaped, Wait Mode is withheld, and every backend ingest site fails
closed on `instrument = 'percussion'` by deliberate interim rule. That interim
was never meant to survive: `music-drums-visibility` says each site fails closed
"until instrument-aware scoring exists (`add-drum-scoring`)", and
`music-drum-kit-view` withholds Wait Mode "until `add-drum-scoring` lands".
This change is that change. It defines what instrument-aware scoring **is** for
a drum part — what counts as the right stroke, what a run's quality means when
notes have no sustain, what a drum score's difficulty is — and then lifts every
interim, one by one, saying for each what "instrument-aware" meant there.

Two recorded deferrals converge here as well. `add-drums-access` left open "how
is a drum score's difficulty assessed?" and `add-unpitched-notation` observed
that the level heuristic's features are keyboard-shaped (ambitus is meaningless,
smallest-value and chords mislead). Difficulty cannot stay open once percussion
runs pay points and rank on the global boards: both are **difficulty-weighted by
construction** — the anti-farming property of `music-play-rewards` rests on the
weight — so an ungraded or absurdly-graded drum corpus would either underpay
every drummer or hand them a farmable weight.

Finally, `add-instrument-context` names the full core loop — gauge and
end-of-session summary on a drum score — as a **general-availability
prerequisite**. This change completes that loop.

## What Changes

**The matcher — what counts as the right stroke**

- Stroke identity is matched **at the kit piece's grain**, using the same
  named-piece table that collapses General MIDI numbers onto lanes
  (`music-drum-kit-view`: hi-hat {42, 46}, snare {37, 38, 40}, ride {51, 53, 59},
  kick {35, 36}; each tom, each accent cymbal and each terminal-bucket number its
  own piece). An electric-snare stroke (40) satisfies a written acoustic snare
  (38); a china never satisfies a written crash. The governing principle: **the
  matcher never demands a distinction the cascade does not draw** — the display
  and the judgment must ask the player for exactly the same thing.
- The one distinction the cascade **does** draw inside a lane — open (46) versus
  closed (42) hi-hat — **shades the verdict but never gates**: the wrong
  articulation binds to the onset and releases the Wait gate, but its timing
  verdict is capped below `perfect`. A player whose hardware cannot produce an
  open hi-hat is never walled out of Wait Mode; a player who can is still
  rewarded for the difference.
- Timing windows and the verdict scale are **reused from keyboard scoring**
  (they are tunable constants, not spec-locked); velocity is **ignored**, as it
  already is for keyboard.
- A percussion run has **no sustain dimension** — a stroke is an attack; its
  note-off is hardware noise. The synchronization blend for percussion is timing
  and correctness only, renormalized to preserve their relative weights. The
  keyboard blend is untouched.
- One stroke identity, every consumer: the Wait gate, the scorer's binding and
  extra-stroke detection, and the pad feedback all consume the same function, so
  the gate can never demand what the scorer would refuse.

**Wait Mode opens for percussion**

- The `music-drum-kit-view` interim ("Wait Mode is not offered … for now") is
  lifted by rename: with `add-drum-input-mapping` in place the gate can be
  satisfied, so Wait Mode is offered for a percussion score exactly as for a
  keyboard one. The gate freezes at each onset and releases when every required
  stroke (of the selected hands/feet) has been struck **while the gate is
  active** — a stroke is an attack with no hold, so the keyboard's held-pitch
  carve-out has no percussion counterpart and none is invented. The
  non-intrusive indicator carries over: the expected pads (and the kick pedal)
  pulse while the gate blocks.

**The gauge, the summary, the replay**

- The score chip, tier feedback and hit sparks run in the cascade (and in the
  percussion notation modes) exactly as in the keyboard modes; hit feedback
  anchors to the note's lane at the hit line, a kick's to the full-width bar.
- The end-of-session summary shows a two-dimension breakdown for a percussion
  run — the sustain row is absent, not zero. The mistake replay renders on the
  percussion scrolling staff (`add-drum-notation-render`) and sounds the
  player's own performance through the percussion channel
  (`add-drum-audio-channel`) — which is why this change is **last** in the
  delivery order.
- Practice (measure-range) runs stay unscored **by construction** for drums
  exactly as for keyboard: the scorer is never armed for a selective run, an
  instrument-agnostic rule that needs no percussion carve-out.

**Difficulty — the two deferrals resolved**

- The crawler's level heuristic becomes **instrument-aware**: for a percussion
  score it estimates from drum-shaped features — stroke density, tempo, fastest
  subdivision, limb simultaneity (simultaneous distinct pieces, two-voice
  writing), kit breadth — instead of the keyboard features (ambitus, leaps,
  accidentals, grand staff), which degenerate to near-zero on unpitched notes
  and would grade every drum part Beginner.
- The estimate lands on the **same** three-level vocabulary
  (beginner/intermediate/advanced), so the existing difficulty weights — one
  shared function for play rewards and the global boards
  (`difficulty_weight_of`) — apply to percussion **without changing keyboard
  weights**, and the farming analysis of `music-play-rewards` (floor,
  diminishing curve, weight, cap) carries over intact.
- Existing percussion rows graded by the keyboard heuristic are **re-graded**;
  only `level_source = heuristic` rows are touched — the provenance rule was
  designed for exactly this overwrite, and source/manual grades are never
  clobbered.

**Backend — every interim ingest site lifted, deliberately, one by one**

- The `music-drums-visibility` interim requirement is lifted by rename. Each
  formerly fail-closed site starts engaging, with "instrument-aware" made
  concrete per site: coverage **engagement** (withheld only to keep the interim
  total — engagement never needed instrument awareness); **play rewards** (the
  quality floor now reads a percussion sync% produced by the drum matcher; the
  weight reads an honestly-graded level); **per-piece leaderboard bests** (fed
  by percussion sub-scores — per-piece boards are drum-against-drum by
  construction); **global season bests** (level-weighted like any piece);
  **streak** (showing up is instrument-agnostic); **daily-access consumption**
  (a percussion open consumes a slot and can be day-locked like any piece — the
  exemption existed only because burning quota on an unplayable score would
  have been wrong).
- **Nothing retroactive**: percussion sessions stored during the interim stay
  inert forever. Engagement is an ingest-time effect keyed on the session's own
  event ids; no pass re-scans history. Idempotence is inherited unchanged —
  awards keyed on the session id, bests monotone.
- The **leaderboard read surface joins the drum-audience enforcement**, the
  obligation `music-drums-visibility` explicitly delegated here: boards are
  keyed by catalog piece, so a percussion piece's board existing is an existence
  oracle. `GetLeaderboard` and the batched `GetMyStandings` answer for a
  percussion piece, to an ineligible caller, exactly as for a piece with no
  board. The **global** boards stay outside the surface: their entries disclose
  players and scores, never pieces.

**Deliberately not included**

- **Velocity / ghost-note / accent scoring.** A real drum dimension, but
  MusicXML drum parts rarely encode dynamics per stroke and e-kit velocity
  curves vary wildly by hardware; scoring it now would judge the pad, not the
  player. Recorded as an open question with the shape a later change would take.
- **Per-instrument global boards.** Percussion and keyboard sub-scores share
  the 0–100 quality scale and the global boards already mix modes with
  different metrics (reaction vs tempo); splitting the boards would halve a
  small community. Accepted with a named risk (see `design.md`).
- **Scoring the pedal hi-hat "chick"** beyond its ordinary-lane interim: it
  matches like any terminal-bucket piece; a dedicated encoding remains
  `music-drum-kit-view`'s recorded candidate.

## Capabilities

### New Capabilities

- `music-drum-scoring`: the percussion matcher — stroke identity at the kit
  piece's grain, the open/closed shading rule, the reused timing windows and
  ignored velocity, the two-dimension blend, and the one-identity-every-consumer
  rule.

### Modified Capabilities

- `music-drums-visibility`: the interim no-scoring requirement is lifted by
  **rename** (its body now asserts the opposite of the old title); the
  enforcement requirement takes on the per-piece leaderboard read surface, as it
  said this change would.
- `music-drum-kit-view`: the Wait-Mode-withheld interim is lifted by **rename**;
  Wait Mode is offered for a percussion score.
- `wait-mode`: a percussion gate requirement (strokes, no hold semantics) is
  added; the non-intrusive indicator extends to the pad strip.
- `performance-scoring`: the run-activation mode list, per-note judgment,
  extra-note rule, sustain scope, synchronization blend and session-result
  record each get their percussion carve-in — sustain is scoped to keyboard,
  stroke identity replaces pitch equality for percussion.
- `gamified-feedback`: the gauge chip and per-note hit feedback extend to the
  cascade and its lane/bar anchors.
- `session-summary`: the summary's dimension breakdown and the mistake replay
  get their percussion shape (no sustain row, percussion staff + channel).
- `hand-selection`: the hidden-hand gate exclusion applies the hands/feet
  classification for percussion — hiding the feet un-awaits the kick.
- `corpus-manifest`: the difficulty heuristic becomes instrument-aware, and
  keyboard-heuristic-graded percussion rows are re-graded under the existing
  provenance rules.

## Impact

**Products**

| Product | Consumes | New |
|---|---|---|
| **Music** (`apps/music`) | the kit-piece table (`drum_kit.dart`), the stroke input path (`add-drum-input-mapping`), the percussion staff and channel (render/audio changes) | the percussion matcher, Wait Mode for drums, gauge/summary/replay in the cascade |
| **Backend** (`backend/music`) | the existing award/best/streak/day-access machinery, unchanged in shape | the six interim lifts, the leaderboard read gate, the heuristic re-grade pass |
| **Crawler** (`crates/score-crawler`) | the parser's unpitched notes and instrument classification | the drum-shaped difficulty estimate |
| **Back-office / ID / Live / Site** | — | untouched |

**Code**

- `apps/music/lib/state/performance_scoring_core.dart` +
  `performance_scoring.dart`: the percussion blend (no sustain) and the
  stroke-identity judgment path.
- `apps/music/lib/state/drum_kit.dart`: the named-piece table becomes the
  matcher's equivalence source (consumed, not duplicated).
- `apps/music/lib/state/player_notifier.dart` / `player_data.dart`: the Wait
  gate's required set over strokes; Wait Mode offered for percussion.
- The summary/replay widgets and the cascade painters: gauge chip, hit anchors,
  two-dimension summary, percussion replay.
- `backend/music/src/play_grpc.rs` (the engagement, award and streak branches),
  `play_module.rs` (the leaderboard-sink branch), `module.rs` (the two
  daily-access exemptions): the six lifts.
- `backend/music/src/leaderboard_grpc.rs` / `leaderboard_module.rs`: the read
  gate (instrument lookup + the existing eligibility predicate).
- `crates/score-crawler/src/difficulty.rs`: the instrument switch and the
  drum feature set; the re-grade pass follows the `backfill.rs` shape.

**Position in the delivery order.** This is the **last** of the eight drum
changes: `add-drum-notation-render` → `add-drum-audio-channel` →
`add-drum-input-mapping` → **`add-drum-scoring`**. It depends **hard** on
`add-drum-input-mapping` (no strokes, no gate, no judgments), and on the render
and audio changes for the mistake replay's staff and sound; it builds on
`add-unpitched-notation` (the General MIDI number per note),
`add-drums-access` (the instrument column and the interim sites it now lifts)
and `add-drum-kit-view` (the kit-piece table and the hands/feet convention).
The audience is inherited from `music-drums-visibility` (`drums.enabled` under
`beta:midi-drums`, backend-enforced) — nothing here re-derives gating.
