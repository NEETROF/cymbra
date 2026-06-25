## Why

Wait Mode today freezes the cascade until the required key is **held for the
whole notated duration**: `requiredNotesAt(t)` returns every note whose window
`[start, start+duration)` contains the playhead, and `advance` blocks while those
notes are not all held. That punishes a learner who plays the right note at the
right moment but lifts before the (often long) note value elapses. Real practice
is about hitting the note **on time**, not sustaining it — sustain/release
accuracy belongs to a future scoring/synchronization system.

## What Changes

- Re-define Wait Mode gating around **note onsets** instead of sustained windows:
  the playhead freezes at each onset where notes begin and releases as soon as the
  required key(s) for that onset have been **pressed** (a key-down), regardless of
  whether they are still held afterwards.
- Validation is **at the right moment**: a press only counts toward the current
  onset gate while the playhead has reached that onset (presses before the
  playhead arrives at the gate do not pre-satisfy it).
- A chord at one onset is satisfied when **all** its required pitches have been
  pressed while the gate is active (not necessarily simultaneously); once
  satisfied, the gate releases and playback continues even if keys are released.
- No sustain/duration enforcement: holding past the press is neither required nor
  penalized. Synchronization/scoring is explicitly **out of scope** (future work).

## Capabilities

### New Capabilities
- `wait-mode`: the practice gate that freezes time-based playback at each note
  onset until the required key(s) are pressed at the right moment, with
  onset-based (not sustained-hold) validation.

### Modified Capabilities
<!-- None. keyboard-display's Three-State Key Feedback keeps referencing "the gate";
     this change defines the gate's onset semantics in the new wait-mode capability
     without altering the feedback requirement's wording. -->

## Impact

- **State/logic**: `apps/music/lib/state/player_notifier.dart` (`advance`,
  onset-gate tracking, latch of pressed pitches for the active gate, reset on gate
  change; `noteOn` participates in validation) and
  `apps/music/lib/state/player_data.dart` (a new onset-based required-set helper
  alongside or replacing the window-based `requiredNotesAt`; a notion of the
  current/next onset).
- **Feedback**: the keyboard's expected/correct colors follow the same onset gate
  (they already read "the gate"), so no change to `keyboard-display` wording.
- **Tests**: unit tests for onset-gate satisfaction (press-then-release advances;
  early press does not pre-satisfy; chord needs all pitches; wrong note keeps
  blocking).
- No Rust/engine, MIDI, or public-API changes. Interacts cleanly with the
  in-flight `hand-selection-filter` change (gate restricted to visible hands) —
  both narrow the same required set along different axes.
