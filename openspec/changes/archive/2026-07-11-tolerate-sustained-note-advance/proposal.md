## Why

In Wait Mode the gate only releases on a fresh key *press* at the onset. A player
who is already holding the required key when the playhead reaches the onset — for
example a repeated or tied pitch, or legato playing where the finger stays down —
never generates a new press, so the gate stays frozen even though the correct
note is sounding. This feels broken: the right key is down, yet the score refuses
to advance until the player releases and re-presses. We want to tolerate a
sustained/held note as satisfying the onset.

## What Changes

- A pitch that is **already held down** at the moment the playhead reaches an
  onset SHALL satisfy that onset's requirement for that pitch (no re-press
  needed), provided the key is still held when the onset becomes active.
- Track the set of currently-held MIDI pitches so the gate can consult "is this
  pitch down right now?" in addition to reacting to press events.
- The existing anti-cheat intent is preserved for the *non-held* case: a key that
  was pressed **and released** before the onset still does not pre-satisfy it —
  only a press that is *sustained through* the onset counts.
- **Re-attack is preserved for repeated notes**: each press counts toward at most
  one onset. A single sustained hold satisfies only the onset it is carried into,
  not a subsequent onset of the same pitch — a repeated pitch still requires a
  fresh attack, so a held key cannot auto-advance through repeats.
- Chord onsets accept any mix of held-through and freshly-pressed pitches: an
  onset releases once every pitch is either already held (and not already consumed
  by a prior onset) or pressed while the gate is active.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `wait-mode`: The "Validation At The Right Moment" requirement is relaxed so that
  a pitch held continuously through an onset counts as satisfying it, while a
  pressed-then-released early key still does not — and each press counts for at
  most one onset, so repeated pitches still require a fresh attack. Chord-onset
  satisfaction is clarified to accept held-through pitches alongside fresh presses.

## Impact

- Wait-Mode gate logic (host-testable core in the Rust engine and/or the Flutter
  player notifier — see design) must consult a live "held pitches" set, not only
  discrete note-on events.
- Requires tracking note-on / note-off to maintain the held-pitch set; note-off
  handling must clear the gate's memory of a held pitch so a stale hold cannot
  satisfy a later onset.
- No public FFI signature change expected; behavior change only. Existing
  wait-mode tests are updated and new tests added for the held-through cases.
