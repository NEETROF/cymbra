## Context

Desktop keyboard input is wired in `_PlayerScreenState._onKey`
(`screens/player_screen.dart`): a static `_keyToPitch` map (A–L → 60–74) calls
`Player.noteOn/noteOff` on key down/up. All input (MIDI, computer keyboard,
on-screen keyboard) converges on those two methods, and `requiredNotesAt(elapsedMs)`
is the Wait Mode gate. `TimedNote` carries `staff` (1 = right hand, 2 = left).

The user wants the desktop keyboard to stop being a tiny piano and instead be a
practice aid: two keys that play the correct expected notes per hand, and two that
fire a near-miss. The on-screen keyboard remains for exact play.

## Goals / Non-Goals

**Goals:**
- Map `a/z` to "play all expected left/right-hand notes" (satisfies the gate).
- Map `q/s` to "play a random near-miss for left/right hand" (does not satisfy).
- Reuse the existing `noteOn/noteOff` path; key-up turns off exactly what key-down
  fired (random varies per press).
- Pure, testable helpers for the per-hand expected set and the near-miss pick.

**Non-Goals:**
- No change to the on-screen keyboard or MIDI.
- No new chord/voicing logic beyond "all expected pitches for the hand".
- No scoring of the near-miss (future scoring capability owns that).

## Decisions

### Decision: Four logical keys A/Z/Q/S, by the user's letters
Map `LogicalKeyboardKey.keyA` (left-correct), `keyZ` (right-correct), `keyQ`
(left-near-miss), `keyS` (right-near-miss). On the user's AZERTY layout these form
a natural 2×2 cluster (a/z on the top row, q/s on the home row; left column = left
hand, top row = correct). Logical keys match the produced letters the user named.
**Alternative considered:** physical (`PhysicalKeyboardKey`) mapping for
layout-independence — rejected because the user specified letters, and logical
keys read clearly.

### Decision: Expected-by-hand helper on PlayerData
Add `Set<int> expectedNotesForHand(double t, {required bool rightHand})` =
pitches of notes whose window contains `t` with `staff == 1` (right) or
`staff >= 2` (left). Built from the same window test as `requiredNotesAt` so the
correct-hand keys satisfy the same gate. **Why:** single source of truth for "what
is due now", split by the staff each note already carries.

### Decision: Pure, injectable near-miss picker
Add a pure function `int nearMissPitch(int expected, {required int lowBound,
required int highBound, required int Function(int) nextRandom})` that returns a
pitch within ±N semitones (N≈3), never equal to `expected`, clamped to
`[lowBound, highBound]` (the displayed keyboard range). Randomness is injected
(`nextRandom`) so unit tests are deterministic; production passes a `Random`. When
several notes are expected, base the near-miss on one of them; the result must not
equal **any** expected pitch (re-pick/offset if it collides). **Why:** keeps the
logic host-testable and guarantees the near-miss never accidentally satisfies the
gate. **Trade-off:** with a very narrow range a valid near-miss might be scarce —
fall back to the nearest in-range non-expected pitch.

### Decision: Track fired pitches per assist key
Keep `Map<LogicalKeyboardKey, Set<int>> _assistPressed`. On key-down compute the
pitches, `noteOn` each, store them; on key-up `noteOff` the stored set and clear
it. **Why:** correct-hand keys may fire chords and near-miss keys pick a fresh
random pitch each press, so key-up must release exactly what key-down fired, not
recompute.

### Decision: Remove `_keyToPitch`; ignore its old keys
Delete the per-pitch map. Non-assist keys return `KeyEventResult.ignored` and
produce no note. The on-screen keyboard path is untouched. **Why:** the user asked
for "only these keys".

## Risks / Trade-offs

- **No expected note when a key is pressed** (between onsets, or piece not
  playing) → the key is a no-op; document and test. The user can still scrub via
  transport.
- **Near-miss accidentally equals an expected pitch** → the picker excludes all
  expected pitches and re-offsets; covered by a unit test.
- **Layout dependence** → logical A/Z/Q/S differ in physical position on QWERTY vs
  AZERTY; acceptable since the user is on AZERTY and named the letters. Revisit
  with physical keys if needed.
- **Existing test churn** → the "computer keyboard fallback presses a key" widget
  test asserts keyA→pitch 60; rewrite it for the assist behavior.
- **Coverage** → expected-by-hand and near-miss picker are pure and unit-tested;
  the key handler is covered by widget tests (correct satisfies gate, near-miss
  does not).

## Migration Plan

Additive behavior change to one input path; no data migration. Removing
`_keyToPitch` changes desktop-keyboard behavior only — MIDI and on-screen play are
unchanged. Rollback restores `_keyToPitch` and `_onKey`. No public API change.

## Open Questions

- **Near-miss spread (N semitones)** and distribution — start with ±1..3 excluding
  0; tune if it feels too easy/hard once the scoring path exists.
- **Velocity** — emit the same default as today's keyboard fallback; revisit with
  the audio/scoring work.
