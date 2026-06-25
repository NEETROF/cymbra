## Context

Wait Mode lives in `Player.advance(dtMs)` (`state/player_notifier.dart:201`). Each
tick it computes `required = requiredNotesAt(elapsedMs)` — every note whose window
`[start, start+duration)` contains the playhead (`player_data.dart:130`) — and
freezes while `!activeNotes.containsAll(required)`. It also clamps `next` to the
next note start so the playhead stops at the upcoming onset.

Two properties of this model are wrong for a learner: (1) because the required set
is the *sustained window*, the key must stay held for the whole note value or the
cascade re-freezes mid-note; (2) validation is purely "is it currently held",
which both forbids early release and allows holding a key down continuously to
sail through. We want onset-based, press-once validation that still demands the
press happen when the playhead reaches the note.

Input already funnels through `Player.noteOn(pitch)`/`noteOff(pitch)` from both
the MIDI stream (`_onMidi`) and the computer-keyboard fallback, so onset detection
has a single choke point.

## Goals / Non-Goals

**Goals:**
- Gate the playhead at each onset; release on key-down of the required pitch(es).
- Allow release immediately after the press without re-blocking.
- Require the press to occur while the playhead is at the onset (no early
  pre-satisfaction).
- Handle chords: accumulate presses for the onset until the full set is pressed.
- Keep feedback (expected/correct) consistent with the new gate with no change to
  the `keyboard-display` Three-State requirement wording.

**Non-Goals:**
- No synchronization/sustain scoring or timing penalty (future capability).
- No change to how notes are derived, to MIDI, or to the engine.
- No change to free playback (Wait Mode off) behavior.

## Decisions

### Decision: Required set becomes the onset set, latched per gate
Add an onset-oriented helper on `PlayerData`, e.g. `Set<int> onsetNotesAt(double
t)` returning the pitches whose `startMs` equals the current gate onset (within
the existing ±1ms tolerance), and `int? nextOnset(double t)`. In the notifier,
track the **active gate**: the onset currently being awaited and a latched
`Set<int> _pressedForGate`. `noteOn(pitch)` adds to `_pressedForGate` when the
gate is active and `pitch` is in the onset set. The gate releases when
`_pressedForGate ⊇ onsetSet`; on release, advance and reset the latch for the next
onset. **Why:** latching decouples "was pressed at this onset" from "is currently
held", which is exactly the press-then-release semantics we want.
**Alternative considered:** keep `containsAll(activeNotes, required)` but shrink
`required` to onsets — rejected because it still reads *current* hold state, so a
quick press+release between ticks would be missed.

### Decision: "At the right moment" = gate must be active when the press lands
A press only counts if it arrives while the playhead has reached the onset (the
gate is active and frozen there). Presses before arrival are ignored for gating —
`noteOn` still updates `activeNotes` for visual feedback but does not fill
`_pressedForGate` until the gate for that onset is active. **Why:** preserves the
timing-exercise intent; prevents "mash the next key early to skip the wait".
**Trade-off:** a player who presses a hair before the cascade visually lands may
need to re-press; acceptable for v1 and revisited when the scoring system adds a
tolerance window.

### Decision: Drive feedback from the same gate, leave Three-State wording intact
The keyboard's expected/correct colors read "the gate". Point that at the onset
set (notes pending at the active gate). After a press validates and the gate
advances, the satisfied note is no longer pending, so it stops showing as
expected — matching the new semantics without editing the `keyboard-display`
Three-State requirement. This also avoids an overlapping delta with the in-flight
`hand-selection-filter` change, which already modifies that requirement.

### Decision: Compose with hand-selection
The onset set is filtered to visible hands (intersect with the hand filter from
`hand-selection-filter` when present). Both changes narrow the same required set
on independent axes; order of merge does not matter because each restricts a
different dimension.

## Risks / Trade-offs

- **Missed press between ticks** → Solved by latching `_pressedForGate` on the
  `noteOn` event rather than sampling `activeNotes` in `advance`.
- **Onset equality/tolerance** → Reuse the existing ±1ms tolerance and the sorted
  `notes` list so chord members at one onset group cleanly; derive `nextOnset`
  from the first start strictly greater than the current gate.
- **Repeated same pitch at consecutive onsets** (e.g. two quarter Cs) → Reset the
  latch when the gate advances so the second onset requires a fresh press; a held
  key from the previous onset must be re-pressed. Document this; it is the correct
  "play it again" behavior and consistent with timing practice.
- **Coverage (≥80%)** → Onset gating is pure logic in the notifier/state; cover
  press-releases-advances, early-press-no-presatisfy, chord-partial-blocks,
  wrong-note-blocks, and Wait-Mode-off-no-gate as unit tests.

## Migration Plan

In-memory behavior change behind the existing `waitMode` flag; no data migration.
Risk is contained to `advance` and the new helpers. Rollback is reverting to the
window-based `requiredNotesAt`/`containsAll` gate. Worth verifying interplay with
the `blocked` flag (used to drive the freeze UI) so it still toggles correctly.

## Open Questions

- Should a brief timing tolerance window (press counts within ±N ms of the onset)
  ship now, or wait for the scoring capability? Lean wait — keep v1 strict and let
  the scoring system own tolerance — but flag for product.
- When Wait Mode is toggled on mid-piece, reset the latch to the onset at the
  current playhead (resolve in implementation).
