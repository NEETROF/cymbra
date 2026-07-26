## Why

Many scores open with several measures of rests (or empty leading measures) before the
first sounding note — it is common for the first note to land in the 3rd or 4th measure.
Today the playhead always starts at absolute time zero, so in every render mode the player
stares at a static screen (or, in Wait Mode, sits through leading measures ticking by in
real time) before anything happens. This lead-in silence is annoying and makes the game feel
broken. Playback should begin at the first real note.

## What Changes

- Introduce an **effective start position** for a piece: the onset of its first sounding
  note (the smallest note onset), skipping leading rests and empty measures.
- Start playback, countdowns, and scored runs from that effective start instead of `0`, in
  **all three render modes** — waterfall (Synthesia), scrolling staff, and engraved
  Partition — and with **Wait Mode either on or off**.
- Apply the effective start everywhere the playhead currently resets to zero: initial score
  load, restart / Retry, hand-switch restart, and loop wrap-around.
- Keep the free-run get-ready countdown behaviour: it arms at the effective start (not only
  at `elapsedMs == 0`) so the first real note still arrives no earlier than GO clears.
- A piece that already starts with a note at time zero (no leading rests) behaves exactly as
  before.

## Capabilities

### New Capabilities
- `first-note-start`: defines the effective start of a piece as its first sounding-note
  onset, requires playback / countdown / scored runs to begin there (skipping leading
  rests), and requires this across all three render modes, both Wait-Mode states, and every
  transport reset (load, restart, retry, hand switch, loop).

### Modified Capabilities
<!-- None. The "beginning of the piece" wording in performance-scoring and gamified-feedback
     still holds — the effective start simply resolves what that instant is — so no existing
     requirement text changes; the interaction is captured as design notes. -->

## Impact

- **State/logic**: [player_notifier.dart](apps/music/lib/state/player_notifier.dart) — the
  places that set `elapsedMs: 0` on load/restart/hand-switch and the loop wrap, plus the
  countdown-arming guard (`state.elapsedMs == 0`). A start-position helper derived from the
  notes list (min `startMs`) in [player_data.dart](apps/music/lib/state/player_data.dart) /
  [notation_playback.dart](apps/music/lib/state/notation_playback.dart).
- **Adjacent behaviours**: the pre-start countdown (gamified-feedback), Wait-Mode onset
  freeze (wait-mode), and metronome beat re-alignment on seek (metronome) must all remain
  correct when the playhead starts at a non-zero position.
- **Tests**: unit tests over the start-position helper and the player notifier (load,
  restart, hand switch, loop, Wait-Mode on/off), keeping Flutter coverage ≥ 80%.
- No changes to the Rust engine, MusicXML parsing, or persisted data.
