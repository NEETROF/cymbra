## Why

Learners practising a piece need a steady reference beat to keep time, especially
while reading the score. The player already knows the tempo and time signature
(it derives them from the MusicXML to drive playback), and the header already
shows a **Tempo** chip — but that chip is read-only and there is no beat
reference. Turning the Tempo chip into a metronome toggle gives a natural,
discoverable beat that is always in sync with the piece, in every view mode.

## What Changes

- **Tap the Tempo chip to toggle a metronome.** The existing `Tempo: NNN` chip in
  the player header becomes a tappable control. Tapping it enables/disables the
  metronome. The chip reflects the on/off state visually.
- **The metronome marks every beat of the measure** — one tick per beat, so the
  number of ticks per measure follows the time signature (4 ticks in 4/4, 3 in
  3/4, 2 in 2/4, …). The first beat (downbeat) is accented so the start of each
  measure is audible. Driven from the score's tempo and time signature so it stays
  in step with the measures.
- **Both audible and visual.** Each beat produces a short audio click *and* a
  visual pulse on the Tempo chip, so it works the same in all three view modes —
  **waterfall (synthesia)**, **staff**, and **partition** — because the indicator
  lives in the shared header.
- **Synchronised with the score playhead.** Beat ticks are derived from the same
  playhead / measure timing that drives playback (honouring the speed multiplier),
  not a free-running timer — so the metronome never drifts from the notes.
- **Silent while paused.** When playback is not running (paused/stopped), the
  metronome emits no ticks; it resumes (still enabled) when playback resumes. The
  enabled/disabled choice itself is a user preference that persists across pause.
- **A synthesized click in the audio engine**, independent of the loaded piano
  SoundFont (so the click sounds the same regardless of which `.sf2` is active and
  is clearly distinct from the piano voices).

## Capabilities

### New Capabilities
- `metronome`: a beat reference for the player — toggled from the header Tempo
  chip, producing a synchronised audible **and** visual beat (accented downbeat)
  derived from the score's tempo and time signature, active in all three view
  modes and silent while playback is paused.

### Modified Capabilities
- `audio-output`: extend the audio FFI/engine with a **metronome click** — a short
  synthesized percussive tick (accented vs. normal) that is mixed into the output
  independently of the piano SoundFont, so a beat click can be sounded without
  using a piano voice.

## Impact

- **Flutter UI**: `lib/screens/player_screen.dart` — the `_TopBar` Tempo `_Chip`
  gains a tap target and active/pulse styling.
- **Flutter state**: `lib/state/player_data.dart` (new `metronomeEnabled` flag and
  beat-tracking) and `lib/state/player_notifier.dart` (`advance()` detects
  beat-boundary crossings against measure timing and fires click + visual pulse;
  toggle action; all-off / reset on stop/seek/pause).
- **Audio seam**: `lib/services/audio_service.dart` (`AudioService` interface +
  `FrbAudioService`) gains a `metronomeClick(accent)` method, kept behind the
  existing injectable seam so it is fakeable in tests.
- **Rust engine**: `rust/src/api/audio.rs` (+ host-testable logic in
  `audio_core.rs`) gains a synthesized click generator and an FFI entry point;
  bridge regenerated (`frb_generated.rs`).
- **Tests**: unit/widget tests for the toggle, beat-crossing detection, pause
  behaviour and chip state; pure click/beat logic kept host-testable to hold
  coverage ≥ 80% for both ecosystems.
- No breaking changes; persisted-settings surface gains one boolean.
